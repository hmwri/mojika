use image::{Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::{collections::HashMap, env, fs::File, path::Path, process::Command};
use std::{io, io::prelude::*};
use BitMap::{Bmp, BmpErr};
mod BitMap {
    use std::io::prelude::*;
    use std::{collections::HashMap, fs::File};
    pub struct Bmp<'a> {
        header: &'a [u8],
        body: &'a [u8],
        header_currentPoint: u32,
        body_currentPoint: u32,
    }

    #[derive(Debug)]
    pub enum BmpErr {
        FileNotFound,
        FaileLoad,
    }
    enum header<'a> {
        name(&'a str),
        size(u8),
    }
    pub fn loadImgFiletoVec(buf: &mut Vec<u8>, filename: String) -> Result<bool, BmpErr> {
        let mut f = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err(BmpErr::FileNotFound),
        };

        match f.read_to_end(buf) {
            Ok(_) => (),
            Err(_) => return Err(BmpErr::FaileLoad),
        };
        println!("Complete reading image file");
        Ok(true)
    }

    impl<'a> Bmp<'a> {
        pub fn new(buf: &'a [u8]) -> Self {
            Bmp {
                header: &buf[0..54],
                body: &buf[54..],
                header_currentPoint: 0,
                body_currentPoint: 0,
            }
        }

        pub fn header_read(&mut self, size: u8) -> &[u8] {
            let start = self.header_currentPoint as usize;
            let end = (self.header_currentPoint + (size as u32)) as usize;
            self.header_currentPoint = end as u32;
            &self.header[start..end]
        }
        pub fn body_read(&mut self, size: u32) -> &[u8] {
            let start = self.body_currentPoint as usize;
            let end = (self.body_currentPoint + size) as usize;
            self.body_currentPoint = end as u32;
            &self.body[start..end]
        }

        pub fn get_light(&mut self) -> i32 {
            let rgb = self.body_read(3);
            (rgb[0] as f32 * 0.07 + rgb[1] as f32 * 0.72 + rgb[2] as f32 * 0.21).round() as i32
        }
        pub fn get_header(&mut self, headerInfo: &mut HashMap<&str, i64>) {
            let headerMap = makeHeaderMap();

            let format = self.header_read(2);
            println!("format:ox{:x}", format[0]);
            println!("format:ox{:x}", format[1]);
            for elem in &headerMap {
                let val = convertU8(self.header_read(if let header::size(s) = &elem[1] {
                    *s
                } else {
                    panic!("Unecpected Error!");
                }));
                let name = if let header::name(n) = &elem[0] {
                    n
                } else {
                    panic!("Unecpected Error!");
                };
                println!("{}:{}", name, val);
                headerInfo.insert(name, val);
            }
        }
    }
    fn convertU8(target: &[u8]) -> i64 {
        let mut result: i64 = 0;
        let bit8: i64 = 256;
        for (i, elem) in (0_u32..).zip(target.iter()) {
            result += (*elem as i64) * bit8.pow(i);
        }
        result
    }

    fn makeHeaderMap<'a>() -> [[header<'a>; 2]; 15] {
        let header_names = [
            [header::name("size"), header::size(4)],
            [header::name("reserveArea1"), header::size(2)],
            [header::name("reserveArea2"), header::size(2)],
            [header::name("headerSize"), header::size(4)],
            [header::name("InformationheaderSize"), header::size(4)],
            [header::name("width"), header::size(4)],
            [header::name("height"), header::size(4)],
            [header::name("plane"), header::size(2)],
            [header::name("color"), header::size(2)],
            [header::name("zipFormat"), header::size(4)],
            [header::name("zipSize"), header::size(4)],
            [header::name("HorizonalResolution"), header::size(4)],
            [header::name("VerticalResolution"), header::size(4)],
            [header::name("colors"), header::size(4)],
            [header::name("importantColors"), header::size(4)],
        ];
        header_names
    }
}

#[derive(Debug)]
struct Density {
    c: char,
    density: u32,
}
fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Mojika Program ver 0.01");
    println!("Programed by Himawari");
    let filename = match args.get(1) {
        Some(a) => a,
        None => panic!("ERROR! Please enter filename"),
    };

    let mut moji = String::new();

    io::stdin()
        .read_line(&mut moji)
        .expect("failed to read line");

    let mut charas: Vec<Density> = Vec::new();
    let count = 0;
    for c in moji.trim().chars() {
        let density = inspectDensity(c);
        let mut i = 0;
        if count != 0 {
            while density < charas[i + 1].density {
                i += 1;
            }
        }
        charas.insert(
            i,
            Density {
                c: c,
                density: density,
            },
        )
    }
    println!("{:?}", charas);
    let len = charas.len();
    let mut buf = Vec::new();
    match BitMap::loadImgFiletoVec(&mut buf, filename.to_string()) {
        Ok(t) => t,
        Err(e) => panic!("{:?}", e),
    };
    let mut header: HashMap<&str, i64> = HashMap::new();
    let mut target = Bmp::new(&buf);
    target.get_header(&mut header);

    let width = header.get("width").unwrap();
    let height = header.get("height").unwrap();

    let mut result: Vec<Vec<char>> = Vec::new();
    for y in 0..*height {
        let mut row: Vec<char> = Vec::new();
        for x in 0..*width {
            let light = target.get_light();
            for i in 0..len {
                if (256 / len * i < light as usize) && ((light as usize) < (256 / len * (i + 1))) {
                    row.push(charas[i].c);
                    break;
                }
            }
        }
        row.push(10 as char);
        result.push(row);
    }
    Command::new("clear").spawn().expect("error");
    for i in (0..result.len()).rev() {
        for elem in &result[i] {
            print!("{}", elem)
        }
    }
}

fn inspectDensity(c: char) -> u32 {
    let filepath = "test.bmp";
    let path = Path::new(filepath);
    let mut image = RgbImage::new(50, 50);

    let font =
        Vec::from(include_bytes!("/Users/taiyuu/rust/mojika/mojika/TanukiMagic.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    let scale = Scale { x: 50.0, y: 50.0 };

    let text = &c.to_string();

    draw_text_mut(
        &mut image,
        image::Rgb([255u8, 255u8, 255u8]),
        0,
        0,
        scale,
        &font,
        text,
    );
    image.save(path).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    match BitMap::loadImgFiletoVec(&mut buf, filepath.to_string()) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    let mut header: HashMap<&str, i64> = HashMap::new();
    let mut target = Bmp::new(&buf);
    target.get_header(&mut header);

    let width = header.get("width").unwrap();
    let height = header.get("height").unwrap();
    let mut density = 0;
    for _ in 0..width * height {
        let rgb = target.body_read(3);
        if rgb[0] == 255 {
            density += 1;
        }
    }
    density
}
