use std::path::{Path, PathBuf};

mod gl_renderer;

use gl_renderer::GLRenderer;
use imgui::*;
use imgui_glow_renderer::{
    glow::{self},
    AutoRenderer,
};
use imgui_sdl2_support::SdlPlatform;
use log::{debug, info, warn};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    video::{GLProfile, Window},
    Sdl,
};

use crate::fc::FC;

pub struct GUI {
    sdl_context: Sdl,
    // Note: this is needed to a stop panic: "expected non-zero GL name"
    _gl_context: sdl2::video::GLContext,
    imgui: Context,
    window: Window,
    platform: SdlPlatform,
    renderer: GLRenderer,
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
    pub fn new() -> GUI {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let gl_attr = video_subsystem.gl_attr();

        gl_attr.set_context_version(3, 3);
        gl_attr.set_context_profile(GLProfile::Core);

        let window = video_subsystem
            .window("rfce", 256 * 2, 240 * 2)
            .allow_highdpi()
            .opengl()
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let gl_context = window.gl_create_context().unwrap();
        window.gl_make_current(&gl_context).unwrap();

        window.subsystem().gl_set_swap_interval(0).unwrap();

        let mut imgui = Context::create();
        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        let platform = SdlPlatform::new(&mut imgui);

        let gl = glow_context(&window);
        let renderer = GLRenderer::new(AutoRenderer::new(gl, &mut imgui).unwrap());

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
            sdl_context,
            imgui,
            _gl_context: gl_context,
            window,
            platform,
            renderer,
            state,
            fc: None,
        }
    }

    pub fn from_file(filename: &Path) -> GUI {
        let mut gui = GUI::new();
        let fc = load_rom(filename).ok();
        gui.fc = fc;
        gui.state.curr_rom_path = filename.to_owned();
        gui
    }

    pub fn run_forever(&mut self) -> () {
        info!("Starting GUI run loop");

        let mut event_pump = self.sdl_context.event_pump().unwrap();
        'running: loop {
            let frame_start = std::time::Instant::now();

            // Handle events
            for event in event_pump.poll_iter() {
                self.platform.handle_event(&mut self.imgui, &event);

                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => self.state.emulator_paused = !self.state.emulator_paused,
                    Event::Window {
                        win_event: WindowEvent::SizeChanged(_, _),
                        ..
                    } => {
                        let (width, height) = self.window.size();
                        self.renderer.update_viewport(width as i32, height as i32);
                    }
                    _ => {}
                }
            }

            // Run the emulator until it's finished rendering (hits scanline 240)
            if let Some(fc) = &mut self.fc {
                if !self.state.emulator_paused {
                    let start = std::time::Instant::now();

                    fc.run_until_render_done();

                    let end = start.elapsed();
                    debug!("Time: {:.2?}", end);

                    let frame_buf = fc.get_frame();
                    self.renderer.update_texture(frame_buf);
                }
            }

            // Handle rendering
            self.platform
                .prepare_frame(&mut self.imgui, &self.window, &event_pump);

            self.handle_imgui();
            let draw_data = self.imgui.render();

            self.renderer.render(draw_data);

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
            self.window.gl_swap_window();
            debug!(
                "Paused for: {:.2?} (f:{:?})",
                frame_time.checked_sub(delta),
                frame_time
            );
            debug!("Frame took: {:?}", frame_start.elapsed());

            if !self.state.continue_running {
                break 'running;
            }
        }
    }

    fn handle_imgui(&mut self) -> () {
        let io = self.imgui.io();

        let [m_x, m_y] = io.mouse_pos;
        let (mouse_x, mouse_y) = (m_x as u32, m_y as u32);
        let (window_w, window_h) = self.window.size();

        // Note: NOT using >= / <=, because io.mouse_pos doesn't update when the mouse
        // is outside of the window, so we use this as the workaround...
        let should_display_menubar =
            mouse_x > 0 && mouse_y > 0 && mouse_x < (window_w - 1) && mouse_y < (window_h - 1);

        let ui = self.imgui.new_frame();

        if should_display_menubar {
            if let Some(menu_bar) = ui.begin_main_menu_bar() {
                if let Some(menu) = ui.begin_menu("File") {
                    if ui.menu_item("Load ROM") {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("rom", &["nes"])
                            .pick_file()
                        {
                            let filename = path.as_path();
                            info!("File: {filename:?}");
                            match load_rom(filename) {
                                Ok(rom) => {
                                    self.fc = Some(rom);
                                    // WARNING: extremely scuffed way of doing things follows.
                                    self.state.curr_rom_path = if self.fc.is_some() {
                                        // idk, seems like great coding conventions to me...
                                        let title = path.file_stem().unwrap().to_str().unwrap();
                                        self.window.set_title(title).unwrap();

                                        path.clone()
                                    } else {
                                        PathBuf::new()
                                    };
                                    self.state.ui_show_error = false;
                                }
                                Err(e) => {
                                    warn!("Error: {}", e);
                                    self.state.ui_show_error = true;
                                    self.state.ui_last_error = Some(e);
                                    self.state.ui_show_error_timer = std::time::Instant::now();
                                }
                            }
                        }
                    }

                    self.state.continue_running = !ui.menu_item("Quit");
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Emu") {
                    ui.menu_item_config("Pause")
                        .build_with_ref(&mut self.state.emulator_paused);
                    if ui.menu_item("Reset") {
                        if let Some(fc) = &mut self.fc {
                            fc.reset();
                        } else {
                            info!("No emulator")
                        }
                    }
                    if ui.menu_item("Hard reset") {
                        if let Some(fc) = &mut self.fc {
                            // We don're actually care if it has a rom loaded or not...
                            if fc.reset_hard().is_err() {
                                info!("No rom loaded");
                            }
                        } else {
                            info!("No emulator")
                        }
                    }
                    menu.end();
                }
                if let Some(menu) = ui.begin_menu("Misc") {
                    ui.menu_item_config("Use integer FPS (60hz)")
                        .build_with_ref(&mut self.state.emulator_60fps);
                    if ui.menu_item("Save screenshot") {
                        if let Some(fc) = &mut self.fc {
                            let width = crate::fc::ppu::PICTURE_WIDTH as u32;
                            let height = crate::fc::ppu::PICTURE_HEIGHT as u32;
                            let frame_buf = fc.get_frame();
                            let img: image::RgbImage =
                                image::RgbImage::from_raw(width, height, frame_buf.to_vec())
                                    .unwrap();

                            // TODO: this is probably really dangerous if something other than a
                            // game is "loaded", or any file named `<romfile>.png` already exists....
                            let img_path = self.state.curr_rom_path.with_extension("png");

                            info!("Saving screenshot: {}", img_path.display());
                            if let Err(e) = img.save(img_path) {
                                warn!("Failed to save screenshot: {}", e);
                            }
                        } else {
                            info!("No emulator")
                        }
                    }
                    menu.end();
                }
                menu_bar.end();
            }
        }

        if self.state.ui_show_error {
            if let Some(e) = &self.state.ui_last_error {
                ui.window("Error")
                    .position_pivot([1.01, 0.0])
                    .position(
                        [window_w as f32, 0.0],
                        Condition::Always,
                    )
                    .movable(false)
                    .resizable(false)
                    .collapsible(false)
                    .title_bar(false)
                    .opened(&mut self.state.ui_show_error)
                    .build(|| {
                        ui.text(format!("Error loading rom: {}", e));
                    });

                if self.state.ui_show_error_timer.elapsed() > std::time::Duration::new(5, 0) {
                    self.state.ui_show_error = false;
                }
            }
        }
    }
}

fn load_rom(filename: &Path) -> Result<Box<FC>, std::io::Error> {
    let mut fc = FC::from_file(filename)?;
    fc.init();
    Ok(fc)
}

fn glow_context(window: &Window) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
    }
}
