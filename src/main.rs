use std::{
    fs,
    fs::File,
    io::{self, BufReader, Read, Write},
};
use substring::Substring;
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

    // Other contents
    let mut i: i32 = 1;

    loop {
        let img_name = get_image(&mut archive, i);
        if img_name.is_empty() {
            break;
        };
        let mut img = match archive.by_name(img_name.as_str()) {
            Ok(file) => file,
            Err(..) => {
                break;
            }
        };

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

fn get_image(archive: &mut ZipArchive<BufReader<File>>, i: i32) -> String {
    let mut file = match archive.by_name(format!("html/{}.html", i).as_str()) {
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
    temp.substring(pos1 + 9, pos2).to_string()
}
