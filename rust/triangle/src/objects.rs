use crate::log;
use crate::shaders::{GradientShader, TextureShader};
use image::{DynamicImage, ImageError};
use std::io::Cursor;
use wasm_bindgen::JsValue;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlTexture, WebGlVertexArrayObject};

pub struct GradientTriangle {
    pub shader: GradientShader,
    pub triangle_vertices: WebGlBuffer,
    pub vao: WebGlVertexArrayObject,
}

impl GradientTriangle {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<Self, JsValue> {
        let shader = GradientShader::new(gl)?;

        let vao = gl
            .create_vertex_array()
            .ok_or_else(|| JsValue::from_str("failed to create vao"))
            .unwrap();
        gl.bind_vertex_array(Some(&vao));

        let triangle_vertices = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from("failed to create buffer"))?;
        gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&triangle_vertices),
        );
        let diam = 1.0;
        let xys = [0.0f32, diam, -diam, -diam, diam, -diam];
        // let xys = [0.0, 0.0, 0.0, 0.001, diam, 0.0];
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::from(xys.as_slice()),
            WebGl2RenderingContext::STATIC_DRAW,
        );

        gl.vertex_attrib_pointer_with_i32(
            shader.sal_xy,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(shader.sal_xy);

        gl.bind_vertex_array(None);

        Ok(Self {
            shader,
            triangle_vertices,
            vao,
        })
    }

    pub fn draw(&self, gl: &WebGl2RenderingContext, mvp: &[f32; 16]) {
        self.shader
            .draw(gl, 0, 3, &self.vao, mvp);
    }
}

//

pub struct SohmahPoster {
    pub shader: TextureShader,
    square_vertices: WebGlBuffer,
    indices: WebGlBuffer,
    index_count: i32,
    tex_id: WebGlTexture,
    vao: WebGlVertexArrayObject,
}

impl SohmahPoster {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<Self, JsValue> {
        let shader = TextureShader::new(gl)?;

        let vao = gl
            .create_vertex_array()
            .ok_or_else(|| JsValue::from_str("failed to create vao"))
            .unwrap();
        gl.bind_vertex_array(Some(&vao));

        let square_vertices = gl.create_buffer().ok_or("failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&square_vertices));
        let diam = 1.0;
        let xys = [-diam, -diam, diam, -diam, -diam, diam, diam, diam];
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::from(xys.as_slice()),
            WebGl2RenderingContext::STATIC_DRAW,
        );

        let indices = gl.create_buffer().ok_or("failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&indices));
        let indices_u8 = [0, 1, 2, 2, 1, 3];
        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &js_sys::Uint8Array::from(indices_u8.as_slice()),
            WebGl2RenderingContext::STATIC_DRAW,
        );

        gl.vertex_attrib_pointer_with_i32(
            shader.sal_xy,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.enable_vertex_attrib_array(shader.sal_xy);

        gl.bind_vertex_array(None);

        let image = sohma_poster().map_err(|e| JsValue::from(format!("{e}")))?;

        let tex_id = texture_from_image(gl, &image)?;

        Ok(Self {
            shader,
            square_vertices,
            indices,
            index_count: indices_u8.len().try_into().unwrap(),
            tex_id,
            vao,
        })
    }

    pub fn draw(&self, gl: &WebGl2RenderingContext, mvp: &[f32; 16]) {
        let tex_index = 0;
        gl.active_texture(WebGl2RenderingContext::TEXTURE0 + tex_index);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.tex_id));

        self.shader.draw(
            gl,
            self.index_count,
            &self.vao,
            mvp,
            tex_index.try_into().unwrap(),
        );
    }
}

//

fn texture_from_image(
    gl: &WebGl2RenderingContext,
    image: &DynamicImage,
) -> Result<WebGlTexture, JsValue> {
    let x = image.color();
    log!("image color space {x:?}");
    let rgb = match &image {
        DynamicImage::ImageRgb8(img) => img.as_flat_samples(),
        _ => return Err(JsValue::from("unable to extract RGB samples from image")),
    };
    let width = image.width();
    let height = image.width();

    let tex_id = gl.create_texture().unwrap();
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&tex_id));
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        WebGl2RenderingContext::RGB.try_into().unwrap(),
        width.try_into().unwrap(),
        height.try_into().unwrap(),
        0,
        WebGl2RenderingContext::RGB,
        WebGl2RenderingContext::UNSIGNED_BYTE,
        Some(rgb.as_slice()),
    )?;
    gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
    Ok(tex_id)
}

pub fn sohma_poster() -> Result<DynamicImage, ImageError> {
    image::ImageReader::new(Cursor::new(include_bytes!("sohma_g_dawling_poster.png")))
        .with_guessed_format()?
        .decode()
}
