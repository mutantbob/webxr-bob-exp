use wasm_bindgen::JsValue;
use web_sys::{
    console, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlUniformLocation,
};

pub struct FlatShader {
    pub program: WebGlProgram,
    pub sal_xy: u32,
    sal_mvp: WebGlUniformLocation,
}

static FLAT_VS: &str = include_str!("flat.vert");
static FLAT_FS: &str = include_str!("flat.frag");

impl FlatShader {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<FlatShader, JsValue> {
        let vertex_shader = load_shader(gl, WebGl2RenderingContext::VERTEX_SHADER, FLAT_VS)?;
        let fragment_shader = load_shader(gl, WebGl2RenderingContext::FRAGMENT_SHADER, FLAT_FS)?;
        let program = gl
            .create_program()
            .ok_or_else(|| JsValue::from("failed to create program"))?;
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);

        let status = gl.get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS);
        if !status.as_bool().unwrap_or(false) {
            return Err(gl
                .get_program_info_log(&program)
                .unwrap_or("shader link failure".into())
                .into());
        }
        let sal_xy = gl.get_attrib_location(&program, "xy").try_into().unwrap();
        let sal_mvp = gl
            .get_uniform_location(&program, "mvp")
            .ok_or_else(|| JsValue::from("missing uniform mvp"))?;
        Ok(Self {
            program,
            sal_xy,
            sal_mvp,
        })
    }

    pub fn draw(
        &self,
        gl: &WebGl2RenderingContext,
        offset: i32,
        vertex_count: i32,
        buffer: &WebGlBuffer,
        projection_matrix: &[f32],
    ) {
        gl.use_program(Some(&self.program));
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer));

        let vao = gl
            .create_vertex_array()
            .ok_or_else(|| JsValue::from_str("failed to create vao"))
            .unwrap();
        gl.bind_vertex_array(Some(&vao));
        gl.vertex_attrib_pointer_with_i32(
            self.sal_xy,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(self.sal_xy);

        gl.bind_vertex_array(Some(&vao));

        gl.uniform_matrix4fv_with_f32_array(Some(&self.sal_mvp), false, projection_matrix);

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, offset, vertex_count);
    }
}

pub fn load_shader(
    gl: &WebGl2RenderingContext,
    type_: u32,
    source: &str,
) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(type_)
        .ok_or_else(|| JsValue::from_str("failed to create shader object"))?;
    gl.shader_source(&shader, source);

    gl.compile_shader(&shader);

    let status = gl.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS);
    console::log_2(&"E ".into(), &status);
    if status.as_bool().unwrap_or(false) {
        Ok(shader)
    } else {
        let message = gl
            .get_shader_info_log(&shader)
            .unwrap_or("shader compile error".into());
        Err(JsValue::from(&message))
    }
}
