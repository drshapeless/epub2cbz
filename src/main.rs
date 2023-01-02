use std::{
    fs,
    fs::File,
    io::{self, BufReader, Read, Write},
};

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
        let mut img = archive.by_name("image/cover.jpg").ok().unwrap();
        let _ = write_image(&mut zip, &mut img, options, 0);
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

        let _ = write_image(&mut zip, &mut img, options, i);

        i += 1;
    }

    let _ = zip.finish();
}

fn write_image(
    zip: &mut ZipWriter<File>,
    img: &mut ZipFile,
    options: FileOptions,
    i: i32,
) -> io::Result<()> {
    let mut buffer = Vec::new();
    img.read_to_end(&mut buffer)?;

    let out_img_name = format!("{:0>3}.jpg", i);
    zip.start_file(out_img_name, options)?;
    zip.write_all(&buffer)?;
    buffer.clear();

    Ok(())
}

fn get_image(archive: &mut ZipArchive<BufReader<File>>, name: String) -> String {
    let mut file = match archive.by_name(name.as_str()) {
        Ok(file) => file,
        Err(..) => {
            return "".to_string();
        }
    };

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let pos1 = contents.find("<img src=").unwrap();
    let pos2 = contents.find(".jpg").unwrap();

    let temp = contents.to_string();
    let ret = temp[pos1 + 13..pos2 + 4].to_string();

    ret
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
