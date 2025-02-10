use crate::gl_thin::HomogeneousGlBuffer;
use crate::shaders::{GradientShader, TextureShader};
use crate::{gl_thin, log};
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

        let diam: f32 = 1.0;
        let xys: [XYRGB; 3] = [
            (0.0f32, diam, 0xff, 0, 0).into(), //
            (-diam, -diam, 0, 0xff, 0).into(), //
            (diam, -diam, 0, 0, 0xff).into(),
        ];

        log!(" xyrgb size {}", size_of::<XYRGB>());

        let triangle_vertices = {
            let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
            XYRGB::load_buffer(
                gl,
                WebGl2RenderingContext::ARRAY_BUFFER,
                &xys,
                WebGl2RenderingContext::STATIC_DRAW,
            );
            buffer
        };

        XYRGB::gl_attr(gl, shader.sal_xy, shader.sal_rgb);

        gl.bind_vertex_array(None);

        Ok(Self {
            shader,
            triangle_vertices,
            vao,
        })
    }

    pub fn draw(&self, gl: &WebGl2RenderingContext, mvp: &[f32; 16]) {
        self.shader.draw(gl, 0, 3, &self.vao, mvp);
    }
}

//

pub struct SohmahPoster {
    pub shader: TextureShader,
    square_vertices: HomogeneousGlBuffer<f32>,
    indices: HomogeneousGlBuffer<u8>,
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

        let diam = 1.0;
        let xys = [-diam, -diam, diam, -diam, -diam, diam, diam, diam];
        let square_vertices = gl_thin::HomogeneousGlBuffer::new_bound(
            gl,
            &xys,
            WebGl2RenderingContext::ARRAY_BUFFER,
            WebGl2RenderingContext::STATIC_DRAW,
        )?;

        let indices_u8 = [0, 1, 2, 2, 1, 3];
        let indices = HomogeneousGlBuffer::new_bound(
            gl,
            &indices_u8,
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            WebGl2RenderingContext::STATIC_DRAW,
        )?;

        square_vertices.vertex_attrib_pointer(gl, shader.sal_xy, 2, false, 0, 0);

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

    pub fn release(self, gl: &WebGl2RenderingContext) {
        self.square_vertices.release(gl);
        self.indices.release(gl);
        self.shader.release(gl);
        gl.delete_vertex_array(Some(&self.vao));
    }
}

//

/// We use this for a heterogenous interleaved GL buffer of vertex data.
/// The X and Y can be used raw, but we should ask GL to "normalize" the r,g,b values.
/// 12 bytes:
///
/// `xxxxyyyyrgb_`
#[repr(C)]
pub struct XYRGB {
    x: f32,
    y: f32,
    r: u8,
    g: u8,
    b: u8,
}

impl XYRGB {
    pub fn load_buffer(gl: &WebGl2RenderingContext, target: u32, xys: &[XYRGB], usage: u32) {
        let raw_slice = {
            let ptr = xys.as_ptr();
            let size = size_of_val(xys);
            log!(" array size {}", size);
            unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), size) }
        };
        gl.buffer_data_with_array_buffer_view(
            target,
            &js_sys::Uint8Array::from(raw_slice).into(),
            usage,
        );
    }

    /// configure the current VAO to pull `vec2 xy;` and `vec3: rgb` from the active buffer
    pub fn gl_attr(gl: &WebGl2RenderingContext, sal_xy: u32, sal_rgb: u32) {
        // heterogeneous buffer layout
        gl.vertex_attrib_pointer_with_i32(
            sal_rgb,
            3,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            true, // convert from u8 to f32: a/255
            size_of::<XYRGB>().try_into().unwrap(),
            (2 * size_of::<f32>()).try_into().unwrap(),
        );
        gl.enable_vertex_attrib_array(sal_rgb);

        gl.vertex_attrib_pointer_with_i32(
            sal_xy,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            size_of::<XYRGB>().try_into().unwrap(),
            0,
        );
        gl.enable_vertex_attrib_array(sal_xy);
    }
}

impl From<(f32, f32, u8, u8, u8)> for XYRGB {
    #[allow(clippy::many_single_char_names)]
    fn from((x, y, r, g, b): (f32, f32, u8, u8, u8)) -> Self {
        Self { x, y, r, g, b }
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
