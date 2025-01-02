use std::time::Duration;

// use imgui::Context;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, render::Canvas, video::Window, Sdl, /* VideoSubsystem */};

pub struct Gui {
    sdl_context: Sdl,
    // video_subsystem: VideoSubsystem,
    // window: Window,
    canvas: Canvas<Window>,
}

impl Gui {
    pub fn new() -> Gui {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("sdl2 test :)", 256 * 2, 240 * 2)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();

        // let mut imgui = Context::create();

        Gui {
            sdl_context,
            // video_subsystem,
            canvas,
        }
    }

    pub fn run_forever(&mut self) -> () {
        self.canvas.clear();
        self.canvas.present();

        let mut event_pump = self.sdl_context.event_pump().unwrap();
        let mut i = 0;
        'running: loop {
            i = (i + 1) % 255;
            self.canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
            self.canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            self.canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
