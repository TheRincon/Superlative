extern crate image;
extern crate imageproc;
extern crate rand;

#[macro_use]
extern crate serde_derive;
extern crate docopt;

use docopt::Docopt;
use std::fs::File;
use std::path::Path;
use std::env;
use std::process::Command;
use rand::{thread_rng, Rng};
use image::{DynamicImage, GenericImage, Pixel};

const USAGE: &'static str = "
Superlative.

Usage:
  superlative --pixelate <image> <outputdir> <prefix>
  superlative <image> <outputdir> <slicedir> <prefix> [--slice]
  superlative --pixelate --slice <image> <outputdir> <slicedir> <prefix>

Options:
  -h --help     Show this screen.
  --image       The input image for pixelator.
  --slicedir    Image files for slicing. 
  --outputdir   Directory for the images.
  --prefix      Prefix for the output .gif FILE.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_delay: Option<i32>,
    flag_pixelate: bool,
    flag_slice: bool,
    arg_image: Option<String>,
    arg_outputdir: Option<String>,
    arg_slicedir: Option<String>,
    arg_prefix: Option<String>,
}

fn gif_wrapper(shim: Option<&str>, del: Option<i32>, dir: &str, output_name: &str, j: i32) {

    fn set_shim(shim: Option<&str>) -> &str {
     match shim {
        Some(x) => x,
        None => "montage"
        }
    }

    fn set_delay(delay: Option<i32>) -> i32 {
    match delay {
        Some(y) => y,
        None => 1
        }
    }

    let output = Command::new("convert")
        .arg("-delay")
        .arg(set_delay(del).to_string())
        .arg("-loop")
        .arg("0")
        .arg(format!("{}{}%d.png[0-{}]", dir, set_shim(shim), j))
        .arg(format!("{}{}.gif", dir, output_name))
        .output()
        .expect("failed to execute convert gif imagemagick");
}

fn path_builder(i: i32, fpath: &str) -> String {
    format!("{}pixelated{}.png", fpath, i)
}

fn gcd(m: i32, n: i32) -> i32 {
   if m == 0 {
      n.abs()
   } else {
      gcd(n % m, m)
   }
}

// fn new_vert_slicer(image_path: &str, s_path: &str, image_total: i32, shim: &str) {
    
//     let mut offset = 0;
//     let mut img1 = DynamicImage::new_rgb8(target.width(), target.height());
//     for image_index in 0..image_total {
//         let mut n = 0;
//         let back = (0..image_total + 1 - image_index).collect::<Vec<_>>();
//         let mut front = (image_total + 1 - image_index..image_total + 1).collect::<Vec<_>>();
//         front.extend(back);
//         for current_index in front.iter() {
//         let slicee = format!("{}{}{}.png", s_path, shim, i);
//         let target = image::open(slicee).expect("Cannot load target image");
//         for k in 0..target.height() {
//             let pos: (i32, i32) = (i, k)
//             imageproc::drawing::draw_filled_circle_mut(&mut img1, pos, 0, rgba);
//         }
//         offset += 1;
//     }
// }

fn slicer_vert(p_path: &str, slice_dir: &str, image_path: &str, image_total: usize) {
    let target = image::open(image_path).expect("Cannot load target image");
    let height = target.height();
    let width = 1;
    for image_index in 0..image_total {
        let mut n = 0;
        let back = (0..image_total - image_index).collect::<Vec<_>>();
        let mut front = (image_total - image_index..image_total).collect::<Vec<_>>();
        front.extend(back);
        for current_index in front.iter() {
            // let mut offset = n + width;
            let mut offset = n;
            let output = Command::new("convert")
                .arg("-crop")
                .arg(format!("{}x{}+{}+0", width, height, offset))
                .arg(format!("{}pixelated{}.png", p_path, current_index))
                .arg(format!("{}sliced{}.png", slice_dir, n))
                .output()
                .expect("failed to execute convert -crop imagemagick");
            n += 1;
        }

        let mut count = image_total + 1;
        let output = Command::new("montage")
                .arg(format!("{}sliced%d.png[0-{}]", slice_dir, image_total))
                .arg("-mode")
                .arg("concatenate")
                .arg("-tile")
                .arg(format!("{}x", count))
                .arg(format!("{}montage{}.png", slice_dir, image_index))
                .output()
                .expect("failed to execute montage imagemagick");
    }
}

fn slicer_horiz(p_path: &str, slice_dir: &str, image_path: &str) {
    let target = image::open(image_path).expect("Cannot load target image");
    let height = target.height();
    let width = target.width();
}

fn pixelator(image_path: &str, p_path: &str) -> i32 {
    let target = image::open(image_path).expect("Cannot load target image");
    let mut rng = rand::thread_rng();
    let mut img1 = DynamicImage::new_rgb8(target.width(), target.height());
    let mut pixel_vec = Vec::with_capacity(target.width() as usize * target.height() as usize);
    // iterate through all the pixels (x,y)
    for cv in 0..target.width() {
        for re in 0..target.height() {
            pixel_vec.push((cv, re));
        }
    }
    // fill all pixels in random order
    rng.shuffle(&mut pixel_vec);
    let mut i = 0;  // pixel index
    let mut j = 0;  // image index
    let mut denom = target.height();
    for (x, z) in pixel_vec {
        // let pos: (u32, u32) = (rng.gen_range(0, target.width()), rng.gen_range(0, target.height()));
        let pos: (u32, u32) = (x, z);
        let pos2: (i32, i32) = (pos.0 as i32, pos.1 as i32);
        let pixel = target.get_pixel(pos.0, pos.1);
        let rgba = pixel;
        i += 1;
        if i % denom == 0 {
            let fpath = path_builder(j, &p_path);
            j += 1;
            img1.save(&mut File::create(&Path::new(&fpath)).unwrap(), image::PNG);
        }
        imageproc::drawing::draw_filled_circle_mut(&mut img1, pos2, 0, rgba);
    }
    let fpath = path_builder(j, &p_path);  // for pixels unfilled for x % 1000
    img1.save(&mut File::create(&Path::new(&fpath)).unwrap(), image::PNG);
    return j;
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.deserialize()).unwrap_or_else(|e| e.exit());
    // println!("{:?}", args);
    let mut j = 0;
    let p_path = args.arg_outputdir.unwrap();
    let output_name = args.arg_prefix.unwrap();
    let slicedir = args.arg_slicedir.unwrap_or_else(|| String::from("Failed to open slicedir"));
    let i_path = args.arg_image.unwrap();
    if args.flag_pixelate && args.flag_slice {
        j = pixelator(&i_path, &p_path);
        slicer_vert(&p_path, &slicedir, &i_path, j as usize);
        let shim = "sliced";
        gif_wrapper(Some(shim), None, &slicedir, &output_name, j);
        println!("{}montage%d.png[0-{}] {}{}.gif", slicedir, j, slicedir, output_name);
        // let output = Command::new("convert")
        //     .arg("-delay")
        //     .arg("1")
        //     .arg("-loop")
        //     .arg("0")
        //     .arg(format!("{}montage%d.png[0-{}]", slicedir, j))
        //     .arg(format!("{}{}.gif", slicedir, output_name))
        //     .output()
        //     .expect("failed to execute convert gif imagemagick");
    } else if args.flag_pixelate {
        j = pixelator(&i_path, &p_path);
        gif_wrapper(Some(shim), None, &p_path, &output_name, j);
        // let output = Command::new("convert")
        //     .arg("-delay")
        //     .arg("5")
        //     .arg("-loop")
        //     .arg("0")
        //     .arg(format!("{}pixelated%d.png[0-{}]", p_path, j))
        //     .arg(format!("{}{}.gif", p_path, output_name))
        //     .output()
        //     .expect("failed to execute convert gif imagemagick");
    } else {
        slicer_vert(&p_path, &slicedir, &i_path, j as usize);
        println!("{}montage%d.png[0-{}] {}{}.gif", slicedir, j, slicedir, output_name);
        gif_wrapper(Some(shim), None, &slicedir, &output_name, j);
        // let output = Command::new("convert")
        //     .arg("-delay")
        //     .arg("5")
        //     .arg("-loop")
        //     .arg("0")
        //     .arg(format!("{}montage%d.png[0-{}]", slicedir, j))
        //     .arg(format!("{}{}.gif", slicedir, output_name))
        //     .output()
        //     .expect("failed to execute convert gif imagemagick");
    }
}