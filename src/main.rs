extern crate image;
extern crate num_complex;
use structopt::StructOpt;

use std::thread;
use std::sync::{mpsc};
use image::{Rgb};


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
fn compute_column(column_number: i32, scale_x: f32, scale_y: f32, height: u32) -> ColumnResult {
    let mut column_pixels = vec![];

    for y in 0..height {
        let cx = y as f32 * scale_x - 1.5;
        let cy = column_number as f32 * scale_y - 1.5;

        let c = num_complex::Complex::new(-0.4, 0.6);
        let mut z = num_complex::Complex::new(cx, cy);

        let mut i: f64 = 0.0;

        while i < 255.0 && z.norm() <= 40.0 {
            z = z * z  + c;
            i += 0.5;
        }

        let pixel = Rgb([0, i as u8, i as u8]);
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
    let scale_x: f32 = 3.0 / img_x as f32;
    let scale_y: f32 = 3.0 / img_x as f32;
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
                channel.send(compute_column((col) as i32, scale_x, scale_y, img_y)).unwrap();
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