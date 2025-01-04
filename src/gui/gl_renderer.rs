use std::rc::Rc;

use imgui_glow_renderer::{
    glow::{self, Context, HasContext, NativeProgram, NativeTexture},
    AutoRenderer,
};
use rgb::bytemuck;

const VERT_SHADER_SOURCE: &str = r#"
    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec2 aTexCoord;

    out vec2 TexCoord;

    void main()
    {
       gl_Position = vec4(aPos, 1.0);
       TexCoord = aTexCoord;
    }
    "#;

const FRAG_SHADER_SOURCE: &str = r#"
    out vec4 FragColor;

    in vec2 TexCoord;

    uniform sampler2D ourTexture;

    void main()
    {
       FragColor = texture(ourTexture, TexCoord);
    }
    "#;

#[rustfmt::skip]
const VERTICES_RECT: [f32; 20] = [
    // positions        // texture coords
    -1.0,  1.0, 0.0,    0.0, 1.0, // top left
     1.0,  1.0, 0.0,    1.0, 1.0, // top right
    -1.0, -1.0, 0.0,    0.0, 0.0, // bottom left
     1.0, -1.0, 0.0,    1.0, 0.0, // bottom right
];

const INDICES: [i32; 6] = [
    0, 1, 2, // first triangle
    1, 3, 2, // second triangle
];

pub struct GLRenderer {
    renderer: AutoRenderer,
    screen_texture: NativeTexture,
    program: NativeProgram,
    buffers: (glow::NativeVertexArray, glow::Buffer, glow::Buffer),
}

impl GLRenderer {
    pub fn new(renderer: AutoRenderer) -> Self {
        let screen_texture = create_texture(renderer.gl_context());

        let gl = renderer.gl_context();

        let buffers = create_vao_vbo_ebo(gl);
        let program = create_program(gl);

        unsafe {
            gl.clear_color(0.1, 0.2, 0.3, 1.0);

            Self {
                renderer,
                program,
                screen_texture,
                buffers,
            }
        }
    }

    fn get_context(&self) -> &Rc<Context> {
        self.renderer.gl_context()
    }

    pub fn update_viewport(&mut self, width: i32, height: i32) -> () {
        let gl = self.get_context();
        unsafe {
            gl.viewport(0, 0, width, height);
        }
    }

    pub fn update_texture(&mut self, data: &[u8]) -> () {
        let width = crate::fc::ppu::PICTURE_WIDTH;
        let height = crate::fc::ppu::PICTURE_HEIGHT;

        unsafe {
            assert!(data.len() == width * height * 3);

            let gl = self.get_context();
            gl.bind_texture(glow::TEXTURE_2D, Some(self.screen_texture));
            gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                0,
                width as i32,
                height as i32,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(data),
            );
        }
    }

    fn render_texture(&mut self) -> () {
        unsafe {
            let gl = self.get_context();
            gl.use_program(Some(self.program));

            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.screen_texture));

            let u_texture_location = gl.get_uniform_location(self.program, "ourTexture").unwrap();
            gl.uniform_1_i32(Some(&u_texture_location), 0);

            let (vao, _, ebo) = self.buffers;
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
        }
    }

    fn clear(&mut self) -> () {
        let gl = self.get_context();
        unsafe {
            gl.clear_color(0.1, 0.1, 0.2, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    pub fn render(&mut self, draw_data: &imgui::DrawData) -> () {
        self.clear();
        self.render_texture();
        self.renderer.render(draw_data).unwrap();
    }
}

fn create_program(gl: &Rc<Context>) -> NativeProgram {
    let shader_sources = [
        (glow::VERTEX_SHADER, VERT_SHADER_SOURCE),
        (glow::FRAGMENT_SHADER, FRAG_SHADER_SOURCE),
    ];

    let mut shaders = Vec::with_capacity(shader_sources.len());

    unsafe {
        let program = gl.create_program().unwrap();

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(
                shader,
                &format!("{}\n{}", "#version 330 core", shader_source),
            );
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        gl.use_program(Some(program));
        program
    }
}

fn create_vao_vbo_ebo(gl: &Context) -> (glow::NativeVertexArray, glow::Buffer, glow::Buffer) {
    unsafe {
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&VERTICES_RECT),
            glow::STATIC_DRAW,
        );

        let ebo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&INDICES),
            glow::STATIC_DRAW,
        );

        gl.vertex_attrib_pointer_f32(
            0,
            3,
            glow::FLOAT,
            false,
            5 * std::mem::size_of::<f32>() as i32,
            0,
        );
        gl.enable_vertex_attrib_array(0);

        gl.vertex_attrib_pointer_f32(
            1,
            2,
            glow::FLOAT,
            false,
            5 * std::mem::size_of::<f32>() as i32,
            3 * std::mem::size_of::<f32>() as i32,
        );
        gl.enable_vertex_attrib_array(1);

        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);

        (vao, vbo, ebo)
    }
}

fn create_texture(gl: &Rc<Context>) -> NativeTexture {
    unsafe {
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGB as i32,
            crate::fc::ppu::PICTURE_WIDTH as i32,
            crate::fc::ppu::PICTURE_HEIGHT as i32,
            0,
            glow::RGB,
            glow::UNSIGNED_BYTE,
            None,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );

        texture
    }
}
