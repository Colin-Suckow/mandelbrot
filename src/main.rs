use pixels::{Pixels, SurfaceTexture};
use winit::{dpi::LogicalSize, event::{Event, VirtualKeyCode}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;
use interpolation::lerp;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const MAX_ITERATIONS: f64 = 32.0;

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
            colors.push(Color::new(
                lerp(&(0x0 as u8), &(0xFF as u8), &(index as f32 / size as f32)),
                lerp(&(0x0 as u8), &(0xFF as u8), &(index as f32 / size as f32)),
                lerp(&(0x55 as u8), &(0xFF as u8), &(index as f32 / size as f32)),
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

fn main() {

    let palette = Palette::generate(MAX_ITERATIONS as usize);
    let mut set: Vec<Color> = Vec::with_capacity(WIDTH * HEIGHT);

    for index in 0..(WIDTH * HEIGHT) {
        let x = index % WIDTH;
        let y = index / WIDTH;

        let x0 = (((1.0 - -2.5) * x as f64) / WIDTH as f64) + -2.5;
        let y0 = (((1.0 - -1.0) * y as f64) / HEIGHT as f64) + -1.0;

        set.push(mandelbrot_calculate_point(x0, y0, &palette));
    }


    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Mandelbrot")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            let frame = pixels.get_frame();
            
            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                pixel.copy_from_slice(&set[i].as_slice());
            }

            if pixels
                .render()
                .map_err(|e| panic!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

         // Handle input events
         if input.update(&event) {

            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize(size.width, size.height);
            }

            // Update internal state and request a redraw
            //world.update();
            window.request_redraw();
        }

        window.request_redraw();
    })
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