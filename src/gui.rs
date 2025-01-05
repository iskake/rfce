use std::path::Path;

mod gl_renderer;

use gl_renderer::GLRenderer;
use imgui::*;
use imgui_glow_renderer::{
    glow::{self},
    AutoRenderer,
};
use imgui_sdl2_support::SdlPlatform;
use log::info;
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    video::{GLProfile, Window},
    Sdl,
};

use crate::fc::FC;

pub struct GUI {
    sdl_context: Sdl,
    imgui: Context,
    _gl_context: sdl2::video::GLContext,
    window: Window,
    platform: SdlPlatform,
    renderer: GLRenderer,
    fc: Option<Box<FC>>,
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

        window.subsystem().gl_set_swap_interval(1).unwrap();


        let mut imgui = Context::create();
        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

        let platform = SdlPlatform::new(&mut imgui);

        let gl = glow_context(&window);
        let renderer = GLRenderer::new(AutoRenderer::new(gl, &mut imgui).unwrap());

        GUI {
            sdl_context,
            imgui,
            _gl_context: gl_context,
            window,
            platform,
            renderer,
            fc: None,
        }
    }

    pub fn from_file(filename: &Path) -> GUI {
        let mut gui = GUI::new();
        let fc = load_rom(filename).ok();
        gui.fc = fc;
        gui
    }

    pub fn run_forever(&mut self) -> () {
        info!("Starting GUI run loop");

        let mut event_pump = self.sdl_context.event_pump().unwrap();
        'running: loop {
            // let frame_start = std::time::Instant::now();

            for event in event_pump.poll_iter() {
                self.platform.handle_event(&mut self.imgui, &event);

                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
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

            if let Some(fc) = &mut self.fc {
                fc.run_until_render_done();
                let frame_buf = fc.get_frame();
                self.renderer.update_texture(frame_buf);
            }

            self.platform
                .prepare_frame(&mut self.imgui, &self.window, &event_pump);

            let continue_running = self.handle_imgui();

            let draw_data = self.imgui.render();

            self.renderer.render(draw_data);

            // let frame_time = std::time::Duration::new(0, 1_000_000_000u32 / 60);
            // let delta = frame_start.elapsed();
            // ::std::thread::sleep(frame_time.abs_diff(delta));
            self.window.gl_swap_window();
            // println!("Paused for: {:2?}", frame_time.abs_diff(delta));

            if !continue_running {
                break 'running;
            }
        }
    }

    fn handle_imgui(&mut self) -> bool {
        let ui = self.imgui.new_frame();

        if let Some(menu_bar) = ui.begin_main_menu_bar() {
            // info!("{:?}", ui.window_size());
            if let Some(menu) = ui.begin_menu("File") {
                if ui.menu_item("Load ROM") {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("rom", &["nes"])
                        .pick_file()
                    {
                        let filename = path.as_path();
                        info!("File: {filename:?}");
                        self.fc = load_rom(filename).ok();
                    }
                }
                if ui.menu_item("Quit") {
                    return false;
                }
                menu.end();
            }
            menu_bar.end();
        }
        true
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
