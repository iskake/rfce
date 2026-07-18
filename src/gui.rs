use std::path::{Path, PathBuf};

// mod gl_renderer;

// use gl_renderer::GLRenderer;
use log::{debug, info};
use sdl3::{
    EventPump, Sdl,
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormat,
    render::{Canvas, FRect, Texture},
    video::Window,
};

use crate::fc::FC;

const SCREEN_WIDTH: u32 = 255;
const SCREEN_HEIGHT: u32 = 240;

pub struct GUI {
    canvas: Canvas<Window>,
    screen_texture: Texture,
    // State of the GUI
    state: GUIState,
    // The emulator istelf.
    fc: Option<Box<FC>>,
}

struct GUIState {
    continue_running: bool,
    emulator_paused: bool,
    emulator_60fps: bool,
    // debugging_view: bool,
    curr_rom_path: PathBuf,
    ui_show_error: bool,
    ui_last_error: Option<std::io::Error>,
    ui_show_error_timer: std::time::Instant,
}

impl GUI {
    pub fn new(sdl_context: Sdl) -> GUI {
        let video_subsystem = sdl_context.video().unwrap();

        // let gl_attr = video_subsystem.gl_attr();

        // gl_attr.set_context_version(3, 3);
        // gl_attr.set_context_profile(GLProfile::Core);

        let window = video_subsystem
            .window("rfce", SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let canvas = window.into_canvas();

        let texture_cretor = canvas.texture_creator();
        let mut screen_texture = texture_cretor
            .create_texture_streaming(PixelFormat::RGB24, 255, 240)
            .unwrap();

        // The rust sdl3 crate doesn't seem to expose SDL_SCALEMODE_PIXELART, so this is the current best.
        screen_texture.set_scale_mode(sdl3::render::ScaleMode::Nearest);

        let state = GUIState {
            continue_running: true,
            emulator_paused: false,
            emulator_60fps: false,
            // debugging_view: false,
            curr_rom_path: PathBuf::new(),
            ui_show_error: false,
            ui_last_error: None,
            ui_show_error_timer: std::time::Instant::now(),
        };

        GUI {
            canvas,
            screen_texture,
            state,
            fc: None,
        }
    }

    pub fn from_file(sdl_context: Sdl, filename: &Path) -> GUI {
        let mut gui = GUI::new(sdl_context);
        let fc = create_fc_from_file(filename).ok();
        gui.fc = fc;
        gui.state.curr_rom_path = filename.to_owned();
        gui
    }

    pub fn run(&mut self, mut event_pump: EventPump) -> Result<(), sdl3::Error> {
        info!("Starting GUI run loop");

        loop {
            for event in event_pump.poll_iter() {
                let quit_event = self.handle_event(event);

                if quit_event {
                    return Ok(());
                }
            }

            let frame_start = std::time::Instant::now();

            // Run the emulator until it's finished rendering (hits scanline 240)
            if let Some(fc) = &mut self.fc {
                if !self.state.emulator_paused {
                    let start = std::time::Instant::now();

                    fc.run_until_render_done();

                    let end = start.elapsed();
                    debug!("Time: {:.2?}", end);

                    let frame_buf = fc.get_frame();
                    update_texture(&mut self.screen_texture, frame_buf);
                }
            }

            let divider = if self.state.emulator_60fps {
                60.0
            } else {
                crate::fc::ppu::FRAMERATE
            };
            let frame_duration = (1_000_000_000.0 / divider) as u32;

            let frame_time = std::time::Duration::new(0, frame_duration);
            let delta = frame_start.elapsed();
            if let Some(time) = frame_time.checked_sub(delta) {
                ::std::thread::sleep(time);
            }

            let (window_w, window_h) = self.canvas.window().size();
            let rect = FRect::new(0.0, 0.0, window_w as f32, window_h as f32);

            self.canvas
                .copy(&self.screen_texture, None, Some(rect))
                .unwrap();
            self.canvas.present();

            debug!(
                "Paused for: {:.2?} (f:{:?})",
                frame_time.checked_sub(delta),
                frame_time
            );
            debug!("Frame took: {:?}", frame_start.elapsed());

            if !self.state.continue_running {
                return Ok(());
            }
            // Ok(())
        }
    }

    /// Handle the given [Event]. Returns `true` if the event was [Event::Quit].
    fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::Quit { .. } => return true,
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => self.pause_emulation(),
            Event::KeyDown {
                keycode: Some(Keycode::O),
                ..
            } => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(".NES ROM file", &["nes"])
                    .pick_file()
                {
                    self.load(&path);
                }
            }
            _ => {}
        }
        false
    }

    fn pause_emulation(&mut self) {
        self.state.emulator_paused = !self.state.emulator_paused
    }

    fn load(&mut self, filename: &Path) {
        self.fc = create_fc_from_file(filename).ok();
    }
}

fn update_texture(screen_texture: &mut Texture, data: &[u8]) {
    let _ = screen_texture.with_lock(None, |buf, _pitch| {
        buf.copy_from_slice(data);
    });
}

fn create_fc_from_file(filename: &Path) -> Result<Box<FC>, std::io::Error> {
    let mut fc = FC::from_file(filename)?;
    fc.init();
    Ok(fc)
}
