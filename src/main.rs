use std::{path::Path, time::SystemTime};
use interpolation::lerp;

const WIDTH: usize = 3840;
const HEIGHT: usize = 2160;
const MAX_ITERATIONS: f64 = 1000.0;

fn main() {

    let palette = Palette::generate(MAX_ITERATIONS as usize);
    let mut set: Vec<Color> = Vec::with_capacity(WIDTH * HEIGHT);

    println!("Calculating mandelbrot set...");
    let start = SystemTime::now();

    for index in 0..(WIDTH * HEIGHT) {
        let x = index % WIDTH;
        let y = index / WIDTH;

        let x0 = (((1.0 - -2.5) * x as f64) / WIDTH as f64) + -2.5;
        let y0 = (((1.0 - -1.0) * y as f64) / HEIGHT as f64) + -1.0;

        set.push(mandelbrot_calculate_point(x0, y0, &palette));
    }

    let calc_time = SystemTime::now().duration_since(start).unwrap();

    println!("Calculated mandelbrot set at {} x {} with {} iterations in {:.2} seconds", WIDTH, HEIGHT, MAX_ITERATIONS as usize, calc_time.as_secs_f32());

    let mut buffer = vec![0; WIDTH * HEIGHT * 4];

    for (i, pixel) in buffer.chunks_exact_mut(4).enumerate() {
        pixel.copy_from_slice(&set[i].as_slice());
    }

    image::save_buffer(&Path::new("mandelbrot.png"), &buffer, WIDTH as u32, HEIGHT as u32, image::ColorType::Rgba8).unwrap();
}

/// Calculate the value of a single colored point on the mandelbrot set
/// x0: scaled x coordinate of pixel (scaled to lie in the Mandelbrot X scale (-2.5, 1))
/// y0: scaled y coordinate of pixel (scaled to lie in the Mandelbrot Y scale (-1, 1))
fn mandelbrot_calculate_point(x0: f64, y0: f64, palette: &Palette) -> Color {
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut iteration: f64 = 0.0;

    while (x*x + y*y) as u64 <= 2^32 && iteration < MAX_ITERATIONS {
        let xtemp = x*x - y*y + x0;
        y = 2.0*x*y + y0;
        x = xtemp;
        iteration += 1.0;
    }

    if iteration < MAX_ITERATIONS {
        let log_zn = (x*x + y*y).log2() / 2.0;
        let nu = (log_zn / (2.0 as f64).log2()).log2() / (2.0 as f64).log2();
        iteration = iteration + 1.0 - nu;
    }

    let c1 = if iteration >= MAX_ITERATIONS {
        &palette.max_color
    } else {
        palette.get_color(((iteration as usize)).min((MAX_ITERATIONS as usize) - 1))
    };
    let c2 = palette.get_color(((iteration as usize) + 1).min((MAX_ITERATIONS as usize) - 1));
    
    c1.interpolate(c2, iteration.fract() as f32)
}

#[derive(Debug)]
struct Color {
    red: u8,
    blue: u8,
    green: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            red: r,
            blue: b,
            green: g,
        }
    }

    fn as_slice(&self) -> [u8; 4] {
        [self.red.clone(), self.green.clone(), self.blue.clone(), 0xFF]
    }

    fn interpolate(&self, other_color: &Color, t: f32) -> Self {
        Self {
            red: lerp(&self.red, &other_color.red, &t),
            green: lerp(&self.green, &other_color.green, &t),
            blue: lerp(&self.blue, &other_color.blue, &t),
        }
    }
}

struct Palette {
    colors: Vec<Color>,
    max_color: Color,
}

impl Palette {
    fn generate(size: usize) -> Self {
        let mut colors = Vec::with_capacity(size);
        for index in 0..size {
            let progress = index as f32 / size as f32;
            let quartic = root(progress, 3);
            colors.push(Color::new(
                lerp(&(0x0 as u8), &(0xFF as u8), &quartic),
                lerp(&(0x0 as u8), &(0xFF as u8), &quartic),
                lerp(&(0x55 as u8), &(0xFF as u8), &quartic),
            ));
        }

        Self {
            colors,
            max_color: Color::new(0, 0, 0)
        }
    }

    fn get_color(&self, index: usize) -> &Color {
        &self.colors[index]
    }
}

fn root(x: f32, n: u32) -> f32 {
    let exp = 1.0 / n as f32;
    if (n & 1) == 0 {
        x.powf(exp)
    } else {
        let absroot = x.abs().powf(exp);
        if x < 0.0 {
            -absroot
        } else {
            absroot
        }
    }
}