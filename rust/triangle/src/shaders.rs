use wasm_bindgen::JsValue;
use web_sys::{
    console, WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation,
    WebGlVertexArrayObject,
};

pub struct GradientShader {
    pub program: WebGlProgram,
    pub sal_xy: u32,
    pub sul_mvp: WebGlUniformLocation,
}

static FLAT_VS: &str = include_str!("flat.vert");
static FLAT_FS: &str = include_str!("flat.frag");

impl GradientShader {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<GradientShader, JsValue> {
        let program = simple_shader_program(gl, FLAT_VS, FLAT_FS)?;
        let sal_xy = gl.get_attrib_location(&program, "xy").try_into().unwrap();
        let sul_mvp = gl
            .get_uniform_location(&program, "mvp")
            .ok_or_else(|| JsValue::from("missing uniform mvp"))?;
        Ok(Self {
            program,
            sal_xy,
            sul_mvp,
        })
    }

    pub fn draw(
        &self,
        gl: &WebGl2RenderingContext,
        offset: i32,
        vertex_count: i32,
        vao: &WebGlVertexArrayObject,
        projection_matrix: &[f32],
    ) {
        gl.use_program(Some(&self.program));

        gl.bind_vertex_array(Some(vao));

        gl.uniform_matrix4fv_with_f32_array(Some(&self.sul_mvp), false, projection_matrix);

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, offset, vertex_count);

        gl.bind_vertex_array(None);
    }
}

//

pub struct TextureShader {
    pub program: WebGlProgram,
    pub sal_xy: u32,
    pub sul_mvp: WebGlUniformLocation,
    pub sul_tex: WebGlUniformLocation,
}

const TEXTURED_VS: &str = include_str!("texture.vert");
const TEXTURED_FS: &str = include_str!("texture.frag");

impl TextureShader {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<Self, JsValue> {
        let program = simple_shader_program(gl, TEXTURED_VS, TEXTURED_FS)?;
        let sal_xy = gl.get_attrib_location(&program, "xy").try_into().unwrap();
        let sul_mvp = gl
            .get_uniform_location(&program, "mvp")
            .ok_or_else(|| JsValue::from("missing uniform mvp"))?;
        let sul_tex = gl
            .get_uniform_location(&program, "tex")
            .ok_or_else(|| JsValue::from("missing uniform tex"))?;
        Ok(Self {
            program,
            sal_xy,
            sul_mvp,
            sul_tex,
        })
    }

    pub fn draw(
        &self,
        gl: &WebGl2RenderingContext,
        index_count: i32,
        vao: &WebGlVertexArrayObject,
        projection_matrix: &[f32],
        texture_id: i32,
    ) {
        gl.use_program(Some(&self.program));

        gl.bind_vertex_array(Some(vao));

        gl.uniform_matrix4fv_with_f32_array(Some(&self.sul_mvp), false, projection_matrix);
        gl.uniform1i(Some(&self.sul_tex), texture_id);

        gl.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLES,
            index_count,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            0,
        );

        gl.bind_vertex_array(None);
    }

    pub fn release(self, gl: &WebGl2RenderingContext) {
        gl.delete_program(Some(&self.program));
    }
}

//

pub fn simple_shader_program(
    gl: &WebGl2RenderingContext,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
) -> Result<WebGlProgram, JsValue> {
    let vertex_shader = load_shader(
        gl,
        WebGl2RenderingContext::VERTEX_SHADER,
        vertex_shader_source,
    )?;
    let fragment_shader = load_shader(
        gl,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        fragment_shader_source,
    )?;
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
    Ok(program)
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
