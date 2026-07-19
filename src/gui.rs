use std::path::{Path, PathBuf};

use log::{debug, info, warn};
use sdl3::{
    EventPump, Sdl,
    event::Event,
    keyboard::Keycode,
    pixels::PixelFormat,
    render::{Canvas, FRect, Texture},
    video::Window,
};

use crate::fc::{FC, ppu};

pub struct GUI {
    canvas: Canvas<Window>,
    screen_texture: Texture,
    state: GUIState,
    fc: Option<Box<FC>>,
}

struct GUIState {
    continue_running: bool,
    emulator_paused: bool,
    emulator_60fps: bool,
    fast_forward: bool,
    frame_advancing: bool,
    // debugging_view: bool,
    curr_rom_path: PathBuf,
    ui_show_error: bool,
    ui_last_error: Option<String>,
    ui_show_error_timer: std::time::Instant,
}

impl GUI {
    pub fn new(sdl_context: Sdl) -> GUI {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "rfce",
                ppu::PICTURE_WIDTH as u32 * 2,
                ppu::PICTURE_HEIGHT as u32 * 2,
            )
            .high_pixel_density()
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let canvas = window.into_canvas();

        let texture_cretor = canvas.texture_creator();
        let mut screen_texture = texture_cretor
            .create_texture_streaming(
                PixelFormat::RGB24,
                ppu::PICTURE_WIDTH as u32,
                ppu::PICTURE_HEIGHT as u32,
            )
            .unwrap();

        // The rust sdl3 crate doesn't seem to expose SDL_SCALEMODE_PIXELART, so this is the current best.
        screen_texture.set_scale_mode(sdl3::render::ScaleMode::Nearest);

        let state = GUIState {
            continue_running: true,
            emulator_paused: false,
            emulator_60fps: false,
            fast_forward: false,
            frame_advancing: false,
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

    /// Create a new GUI, loading `filename` into a new emulator.
    pub fn from_file(sdl_context: Sdl, filename: &Path) -> GUI {
        let mut gui = GUI::new(sdl_context);
        let fc = create_fc_from_file(filename).ok();
        gui.fc = fc;
        gui.state.curr_rom_path = filename.to_owned();
        gui
    }

    /// Run the GUI until it is manually stopped or an error occurs.
    pub fn run(&mut self, mut event_pump: EventPump) -> Result<(), sdl3::Error> {
        info!("Starting GUI run loop");

        loop {
            for event in event_pump.poll_iter() {
                let quit_event = self.handle_event(event);

                if quit_event {
                    return Ok(());
                }
            }

            self.run_frame();

            if !self.state.continue_running {
                return Ok(());
            }
        }
    }

    /// Run the emulator for one frame (~16.6ms).
    fn run_frame(&mut self) {
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

        if self.state.frame_advancing {
            self.state.frame_advancing = false;
            self.state.emulator_paused = true;
        }

        let divider = if self.state.emulator_60fps {
            60.0
        } else {
            crate::fc::ppu::FRAMERATE
        };
        let frame_duration = (1_000_000_000.0 / divider) as u32;

        let frame_time = std::time::Duration::new(0, frame_duration);
        let delta = frame_start.elapsed();
        if !self.state.fast_forward
            && let Some(time) = frame_time.checked_sub(delta)
        {
            ::std::thread::sleep(time);
        }

        let (window_w, window_h) = self.canvas.window().size_in_pixels();
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
        debug!("(Pre-sleep: {:?})", delta);
    }

    /// Handle the given [Event]. Returns `true` if the event was [Event::Quit].
    fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::Quit { .. } => return true,
            Event::KeyDown { keycode: Some(Keycode::_1), .. } => self.set_scale(1),
            Event::KeyDown { keycode: Some(Keycode::_2), .. } => self.set_scale(2),
            Event::KeyDown { keycode: Some(Keycode::_3), .. } => self.set_scale(3),
            Event::KeyDown { keycode: Some(Keycode::_4), .. } => self.set_scale(4),
            Event::KeyDown { keycode: Some(Keycode::_5), .. } => self.set_scale(5),
            Event::KeyDown {
                keycode: Some(Keycode::Escape | Keycode::P),
                ..
            } => self.pause_emulation(),
            Event::KeyDown {
                keycode: Some(Keycode::Tab),
                ..
            } => self.enable_fast_forward(),
            Event::KeyUp {
                keycode: Some(Keycode::Tab),
                ..
            } => self.disable_fast_forward(),
            Event::KeyDown {
                keycode: Some(Keycode::R),
                ..
            } => {
                if let Some(f) = &mut self.fc {
                    info!("Soft reset");
                    f.reset();
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::RightBracket),
                ..
            } => {
                if let Some(f) = &mut self.fc {
                    info!("Frame advance");
                    self.state.emulator_paused = false;
                    self.state.frame_advancing = true;
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::H),
                ..
            } => {
                if let Some(f) = &mut self.fc {
                    info!("Hard reset");
                    if let Err(_) = f.reset_hard() {
                        warn!("Failed to hard reset: no rom loaded")
                    }
                }
            }
            Event::KeyDown {
                keycode: Some(Keycode::PrintScreen),
                ..
            } => {
                if let Some(fc) = &mut self.fc {
                    let width = ppu::PICTURE_WIDTH as u32;
                    let height = ppu::PICTURE_HEIGHT as u32;
                    let frame_buf = fc.get_frame();
                    let img: image::RgbImage =
                        image::RgbImage::from_raw(width, height, frame_buf.to_vec()).unwrap();

                    // TODO: this is probably really dangerous if something other than a
                    // game is "loaded", or any file named `<romfile>.png` already exists....
                    let img_path = self.state.curr_rom_path.with_extension("png");

                    info!("Saving screenshot: {}", img_path.display());
                    if let Err(e) = img.save(img_path) {
                        warn!("Failed to save screenshot: {}", e);
                    }
                } else {
                    warn!("Failed to save screenshot: no rom loaded")
                }
            }
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

    /// Pause emulator
    fn pause_emulation(&mut self) {
        self.state.emulator_paused = !self.state.emulator_paused
    }

    /// Create a new emulator with the given file
    fn load(&mut self, filename: &Path) {
        self.fc = create_fc_from_file(filename).ok();
        self.state.curr_rom_path = filename.to_path_buf();
    }

    fn enable_fast_forward(&mut self) {
        self.state.fast_forward = true;
    }

    fn disable_fast_forward(&mut self) {
        self.state.fast_forward = false;
    }

    fn set_scale(&mut self, scale: u32) {
        let window = self.canvas.window_mut();
        window.set_size(ppu::PICTURE_WIDTH as u32 * scale, ppu::PICTURE_HEIGHT as u32 * scale).unwrap();
    }
}

/// Update the screen texture.
fn update_texture(screen_texture: &mut Texture, data: &[u8]) {
    let _ = screen_texture.with_lock(None, |buf, _pitch| {
        buf.copy_from_slice(data);
    });
}

/// Create a new [FC] struct from the given file.
fn create_fc_from_file(filename: &Path) -> Result<Box<FC>, std::io::Error> {
    let mut fc = FC::from_file(filename)?;
    fc.init();
    Ok(fc)
}
