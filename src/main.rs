use std::{
    borrow::Cow,
    collections::HashMap,
    convert::Infallible,
    fs,
    fs::File,
    io::{self, BufReader, Read, Write},
    path::Path,
};

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use zip::{read::ZipFile, write::FileOptions, ZipArchive, ZipWriter};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <filename>", args[0]);
        return;
    }

    let fname = std::path::Path::new(&*args[1]);
    let file = fs::File::open(fname).unwrap();
    let reader = BufReader::new(file);

    let mut archive = zip::ZipArchive::new(reader).unwrap();

    let fname2 = fname.with_extension("cbz");
    let file2 = fs::File::create(fname2).unwrap();
    let mut zip = zip::ZipWriter::new(file2);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    // Cover page
    {
        let my_img_name = String::from("image/cover.jpg");
        let mut img = archive.by_name(my_img_name.as_str()).ok().unwrap();
        let _ = write_image(&mut zip, &mut img, options, 0, my_img_name);
    }

    // Open the .opf file.

    let contents = get_opf_string(&mut archive);

    let mut i: i32 = 1;

    loop {
        let ret = contents.find(format!("<item id=\"Page_{}\"", i).as_str());
        if ret == None {
            break;
        }

        let occur = ret.unwrap();
        let start = contents[occur..].to_string().find("href").unwrap();
        let end = contents[occur..].to_string().find("media-type").unwrap();

        let name = contents[occur + start + 6..occur + end - 2].to_string();
        let img_name = get_image(&mut archive, name);

        let mut img = archive.by_name(img_name.as_str()).ok().unwrap();

        let _ = write_image(&mut zip, &mut img, options, i, img_name);

        i += 1;
    }

    let _ = zip.finish();
}

fn write_image(
    zip: &mut ZipWriter<File>,
    img: &mut ZipFile,
    options: FileOptions,
    i: i32,
    img_name: String,
) -> io::Result<()> {
    let mut buffer = Vec::new();
    img.read_to_end(&mut buffer)?;

    let extension = Path::new(img_name.as_str()).extension().unwrap();

    let out_img_name = format!("{:0>3}.{}", i, extension.to_str().unwrap());
    zip.start_file(out_img_name, options)?;
    zip.write_all(&buffer)?;
    buffer.clear();

    Ok(())
}

fn get_image(archive: &mut ZipArchive<BufReader<File>>, name: String) -> String {
    let mut file = match archive.by_name(name.as_str()) {
        Ok(file) => file,
        Err(..) => {
            return String::new();
        }
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let mut reader = Reader::from_str(contents.as_str());
    reader.trim_text(true);
    reader.expand_empty_elements(true);
    let mut buf = Vec::new();

    let mut shits: HashMap<String, String> = HashMap::new();

    loop {
        let event = reader.read_event_into(&mut buf).unwrap();

        match event {
            Event::Start(element) => {
                match element.name().as_ref() {
                    b"img" => {
                        shits = element
                        .attributes()
                        .map(|attr_result| {
                            match attr_result {
                            Ok(a) => {
                                let key = reader.decoder().decode(a.key.local_name().as_ref())
                                    .or_else(|err| {
                                        dbg!("unable to read key in img attributes {:?}, error {:?}", &a, err);
                                        Ok::<Cow<'_, str>, Infallible>(std::borrow::Cow::from(""))
                                    })
                                    .unwrap()
                                    .to_string();
                                let value = a.decode_and_unescape_value(&reader).or_else(|err| {
                                    dbg!("unable to read key in img attribute {:?}, error {:?}", &a, err);
                                    Ok::<Cow<'_, str>, Infallible>(std::borrow::Cow::from(""))
                                }).unwrap().to_string();
                                (key, value)
                            },
                            Err(err) => {
                                dbg!("unable to read key in img, err: {:?}", err);
                                (String::new(), String::new())
                            }
                            }

                        }).collect();
                        reader.read_to_end(element.name()).unwrap();
                    }

                    _ => (),
                }
            }
            Event::Eof => break,
            _ => (),
        }
    }

    let mypath = Path::new(shits.get("src").unwrap());
    // println!("{:?}", mypath.strip_prefix("../").unwrap());

    String::from(mypath.strip_prefix("../").unwrap().to_str().unwrap())
}

fn get_opf_string(archive: &mut ZipArchive<BufReader<File>>) -> String {
    let mut opf = match archive.by_name("vol.opf") {
        Ok(file) => file,
        Err(..) => panic!("shit!"),
    };

    let mut contents = String::new();
    opf.read_to_string(&mut contents).unwrap().to_string();
    contents
}
