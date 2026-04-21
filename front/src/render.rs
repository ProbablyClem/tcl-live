use crate::webgl::WebGlRenderer;
use web_sys::WebGl2RenderingContext;

impl WebGlRenderer {
    pub fn render(&self) {
        let gl = &self.gl;

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        gl.use_program(Some(&self.program));
        gl.bind_vertex_array(Some(&self.vao));

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);

        gl.bind_vertex_array(None);
    }
}
