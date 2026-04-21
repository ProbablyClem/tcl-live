use wasm_bindgen::prelude::*;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader,
    WebGlVertexArrayObject,
};

pub struct WebGlRenderer {
    pub gl: WebGl2RenderingContext,
    pub program: WebGlProgram,
    pub vao: WebGlVertexArrayObject,
    _vbo: WebGlBuffer, // keep alive
}

impl WebGlRenderer {
    pub fn new() -> Self {
        let gl = init_webgl_context().expect("Failed to init webgl_context");
        let program = init_program(&gl);
        gl.use_program(Some(&program));

        let vertices: [f32; 6] = [0.0, 0.5, -0.5, -0.5, 0.5, -0.5];

        let vbo = gl.create_buffer().expect("Failed to create buffer");
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

        unsafe {
            let vert_array = js_sys::Float32Array::view(&vertices);
            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        let vao = gl.create_vertex_array().expect("Could not create VAO");
        gl.bind_vertex_array(Some(&vao));

        let position = gl.get_attrib_location(&program, "position") as u32;

        gl.enable_vertex_attrib_array(position);
        gl.vertex_attrib_pointer_with_i32(position, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
        gl.bind_vertex_array(None);
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);

        Self {
            gl,
            program,
            vao,
            _vbo: vbo,
        }
    }
}

pub fn init_webgl_context() -> Result<WebGl2RenderingContext, JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let canvas: HtmlCanvasElement = document
        .get_element_by_id("canvas")
        .ok_or("No canvas")?
        .dyn_into()?;

    let gl: WebGl2RenderingContext = canvas
        .get_context("webgl2")?
        .ok_or("No WebGL2")?
        .dyn_into()?;

    Ok(gl)
}

pub fn init_program(gl: &WebGl2RenderingContext) -> WebGlProgram {
    let vertex_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::VERTEX_SHADER,
        r#"
            attribute vec2 position;
            void main() {
                    gl_Position = vec4(position, 0.0, 1.0);
                }
                "#,
    )
    .expect("Failed to build vertex shader");

    let fragment_shader = compile_shader(
        &gl,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r#"
            precision mediump float;
            void main() {
                    gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0);
                }
                "#,
    )
    .expect("Failed to build fragment_shader");
    link_program(&gl, &vertex_shader, &fragment_shader).expect("Failed to link program to gl")
}

fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or("Unable to create shader object")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl.get_shader_info_log(&shader).unwrap_or_default())
    }
}

fn link_program(
    gl: &WebGl2RenderingContext,
    vertex_shader: &WebGlShader,
    fragment_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or("Unable to create shader program")?;
    gl.attach_shader(&program, vertex_shader);
    gl.attach_shader(&program, fragment_shader);
    gl.link_program(&program);
    if gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl.get_program_info_log(&program).unwrap_or_default())
    }
}
