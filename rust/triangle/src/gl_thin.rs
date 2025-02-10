use js_sys::{Float32Array, Object, Uint8Array};
use std::marker::PhantomData;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

/// A wrapper to make working with homogeneous GL data buffers easier.
/// If you mix types in your GL buffer, this class will not be useful.
pub struct HomogeneousGlBuffer<T> {
    pub buffer: WebGlBuffer,
    pub target: u32,
    phantom: PhantomData<T>,
}

impl<T> HomogeneousGlBuffer<T>
where
    [T]: ToBufferPayload,
{
    /// The new buffer object remains bound to `target` which is useful when configuring VAOs.
    /// # params
    /// * `target` -  probably [`WebGl2RenderingContext::ARRAY_BUFFER`] or  [`WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER`]
    /// * `usage` - probably [`WebGl2RenderingContext::STATIC_DRAW`]
    pub fn new_bound(
        gl: &WebGl2RenderingContext,
        payload: &[T],
        target: u32,
        usage: u32,
    ) -> Result<Self, &'static str> {
        let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
        gl.bind_buffer(target, Some(&buffer));
        gl.buffer_data_with_array_buffer_view(
            target,
            &ToBufferPayload::to_buffer_payload(payload),
            usage,
        );

        Ok(Self {
            buffer,
            target,
            phantom: PhantomData,
        })
    }

    pub fn replace(&self, gl: &WebGl2RenderingContext, payload: &[T], usage: u32) {
        gl.bind_buffer(self.target, Some(&self.buffer));
        gl.buffer_data_with_array_buffer_view(
            self.target,
            &ToBufferPayload::to_buffer_payload(payload),
            usage,
        );
    }
}

impl<T> HomogeneousGlBuffer<T>
where
    T: GlType,
{
    ///  This is unsuitable for use with mixed-type rows because it assumes all the attributes in the buffer have the same type `T`.
    /// # params
    ///
    /// `stride` - the count of elements in each row of the array.
    ///  Will be multiplied by the byte-size of the element type to provide GL with a byte size.
    ///
    /// `offset` - the position of this attribute within the row.
    ///  Will be multiplied by the byte-size of the element type to provide GL with a byte offset.
    ///
    /// # see also
    /// [MDN docs with example](https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/vertexAttribPointer)
    pub fn vertex_attrib_pointer(
        &self,
        gl: &WebGl2RenderingContext,
        shader_attribute_location: u32,
        element_size: i32,
        normalized: bool,
        stride: i32,
        offset: i32,
    ) {
        #[allow(clippy::cast_possible_wrap)]
        gl.vertex_attrib_pointer_with_i32(
            shader_attribute_location,
            element_size,
            <T as GlType>::my_type(),
            normalized,
            stride * size_of::<T>() as i32,
            offset * size_of::<T>() as i32,
        );

        gl.enable_vertex_attrib_array(shader_attribute_location);
    }
}

impl<T> HomogeneousGlBuffer<T> {
    pub fn release(self, gl: &WebGl2RenderingContext) {
        gl.delete_buffer(Some(&self.buffer));
    }
}

//

pub trait ToBufferPayload {
    fn to_buffer_payload(&self) -> Object;
}

impl ToBufferPayload for [f32] {
    fn to_buffer_payload(&self) -> Object {
        Float32Array::from(self).into()
    }
}

impl ToBufferPayload for [u8] {
    fn to_buffer_payload(&self) -> Object {
        Uint8Array::from(self).into()
    }
}

//

pub trait GlType {
    fn my_type() -> u32;
}

impl GlType for f32 {
    fn my_type() -> u32 {
        WebGl2RenderingContext::FLOAT
    }
}
