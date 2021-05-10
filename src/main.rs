extern crate image;
extern crate num_complex;
use structopt::StructOpt;

use std::thread;
use std::sync::{mpsc};
use image::{Rgb, Pixel};


use std::sync::mpsc::{Sender, Receiver};

#[derive(Debug, StructOpt)]
#[structopt(name = "Mandelbrot", about = "A threaded mandelbrot implementation")]
struct Args {
    /// Height of the image that we would like to create
    #[structopt(short)]
    height: u32,

    /// Width of the image that we would like to create
    #[structopt(short)]
    width: u32,

    /// Number of threads that you would like to use - please check how many threads that your cpu has
    #[structopt(short)]
    threads: u32,

    /// File name to save the fractal as
    #[structopt(short)]
    file_name: String,
}


/// The column result is a computed column that is sent to the channel
struct ColumnResult {
    column_number: i32,
    column_pixels: Vec<Rgb<u8>>,
}

// The task that is being threaded
fn compute_column(column_number: i32, height: u32, width: u32) -> ColumnResult {
    let mut column_pixels = vec![];
    let cx = -0.9;
    let cy = 0.27015;
    let iterations = 1000;

    for y in 0..height {
        let inner_height = height as f32;
        let inner_width = width as f32;
        let inner_y = y as f32;
        let inner_x = column_number as f32;

        let mut zx = 3.0 * (inner_x - 0.5 * inner_width) / (inner_width);
        let mut zy = 2.0 * (inner_y - 0.5 * inner_height) / (inner_height);

        let mut i = iterations;

        while zx * zx + zy * zy < 4.0 && i > 1 {
            let tmp = zx * zx - zy * zy + cx;
            zy = 2.0 * zx * zy + cy;
            zx = tmp;
            i -= 1;
        }

        // guesswork to make the rgb color values look okay
        let r = (i << 3) as u8;
        let g = (i << 5) as u8;
        let b = (i * 4) as u8;
        let pixel = Rgb::from_channels(r, g, b, 0);

        column_pixels.push(pixel);
    }

    ColumnResult {
        column_number,
        column_pixels,
    }
}

fn main() {
    // Constants that are passed in system arguments
    let args = Args::from_args();
    let nthreads = args.threads;
    let img_x = args.width;
    let img_y = args.height;
    let file_name = args.file_name;

    let (writer, reader): (Sender<ColumnResult>, Receiver<ColumnResult>) = mpsc::channel();

    let c = img_x / nthreads;

    for n in 0..nthreads {
        // The sender endpoint can be copied
        let channel = writer.clone();

        // Spawn a threads where each thread computes columns of the image
        thread::spawn(move || {
            // Each thread queues a message in the channel
            for col in n*c..(n + 1)*c  {
                channel.send(compute_column((col) as i32,  img_y, img_x)).unwrap();
            }
        });
    }

    let mut imgbuf = image::ImageBuffer::new(img_x, img_y);

    // Here, all the messages are collected
    for _ in 0..img_x {
        let result = &reader.recv().unwrap();
        let index = result.column_number;
        let pixels = &result.column_pixels;
        for (i, pixel) in pixels.iter().enumerate() {
            imgbuf.put_pixel(index as u32, i as u32, *pixel);
        }
    }

    // Save the file
    imgbuf.save(format!("{}.png", file_name)).unwrap();
}