#![cfg(web_sys_unstable_apis)]

#[macro_use]
mod utils;
mod shaders;
#[cfg(test)]
mod test;

use crate::shaders::FlatShader;
use cgmath::{ElementWise, Transform};
use js_sys::{Date, Float32Array, Object, Promise, Reflect};
use std::cell::RefCell;
use std::rc::Rc;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, sierpinski!");
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
pub(crate) use log;

mod helper {
    use wasm_bindgen::JsValue;
    use web_sys::{Document, Window};

    pub fn window() -> Result<Window, JsValue> {
        crate::window().ok_or_else(|| JsValue::from("no window"))
    }

    pub fn document() -> Result<Document, JsValue> {
        window()?
            .document()
            .ok_or_else(|| JsValue::from("no document"))
    }

    pub fn append_to_document(message: &str) -> Result<(), JsValue> {
        let document = document()?;
        let _ = document
            .body()
            .unwrap()
            .append_child(&document.create_text_node(message));
        Ok(())
    }
}

fn request_animation_frame_xr(session: &XrSession, f: &Closure<dyn FnMut(f64, XrFrame)>) -> u32 {
    // This turns the Closure into a js_sys::Function
    // See https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#casting-a-closure-to-a-js_sysfunction
    session.request_animation_frame(f.as_ref().unchecked_ref())
}

pub fn create_webgl_context(xr_mode: bool) -> Result<WebGl2RenderingContext, JsValue> {
    fn jserr(msg: &str) -> JsValue {
        msg.into()
    }
    let canvas = helper::document()?
        .get_element_by_id("canvas")
        .ok_or_else(|| jserr("couldn't find canvas"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| jserr("wasn't HtmlCanvasElement"))?;

    let gl: WebGl2RenderingContext = if xr_mode {
        let gl_attribs = Object::new();
        Reflect::set(
            &gl_attribs,
            &JsValue::from_str("xrCompatible"),
            &JsValue::TRUE,
        )
        .unwrap();

        canvas
            .get_context_with_context_options("webgl2", &gl_attribs)?
            .unwrap()
            .dyn_into()?
    } else {
        canvas.get_context("webgl2")?.unwrap().dyn_into()?
    };

    Ok(gl)
}

#[wasm_bindgen]
extern "C" {
    fn debug_new_layer(session: &XrSession, ctx: &WebGl2RenderingContext) -> XrWebGlLayer;
}

struct DrawLogic {
    flat_shader: FlatShader,
    triangle_vertices: WebGlBuffer,
}

impl DrawLogic {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<Self, JsValue> {
        let flat_shader = FlatShader::new(gl)?;
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
        Ok(Self {
            flat_shader,
            triangle_vertices,
        })
    }

    fn blue(&self) -> f32 {
        //log!("compute b");
        let x = Date::now();
        //log!("x {x:?}");
        (x % 2000.0) as f32 / 2000.0
    }

    pub fn draw(&self, gl: &WebGl2RenderingContext) {
        //gl.viewport(0, 0, 200, 200);
        gl.clear_color(0.0, 1.0, self.blue(), 1.0);
        gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        let identity = [
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        ];
        self.flat_shader
            .draw(gl, 0, 3, &self.triangle_vertices, &identity)
    }

    pub fn draw_xr(
        &self,
        gl: &WebGl2RenderingContext,
        _timestamp: f64,
        frame: &XrFrame,
        viewer_ref_space: &XrReferenceSpace,
        session: &XrSession,
    ) {
        // log!("get pose");
        let viewer_pose = frame.get_viewer_pose(viewer_ref_space);
        let Some(viewer_pose) = viewer_pose else {
            return;
        };
        let gl_layer = session.render_state().base_layer().unwrap();

        gl.bind_framebuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            gl_layer.framebuffer().as_ref(),
        );
        gl.clear_color(0.0, 1.0, self.blue(), 1.0);
        gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        for view in viewer_pose.views() {
            // console::log_2(&"view ".into(), &view);
            let view = &XrView::from(view);
            let viewport = gl_layer.get_viewport(view).unwrap();
            // console::log_2(&"viewport ".into(), &viewport);
            gl.viewport(
                viewport.x(),
                viewport.y(),
                viewport.width(),
                viewport.height(),
            );
            self.draw_xr_single(gl, view);
        }
    }

    pub fn draw_xr_single(&self, gl: &WebGl2RenderingContext, xr_view: &XrView) {
        let pv = projection_view_for(xr_view);

        {
            let offset = cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 0.0, -1.0));
            let scale = cgmath::Matrix4::from_scale(0.2);
            let model = offset * scale;

            let mvp = pv * model;
            let mvp_flat: &[f32; 16] = mvp.as_ref();

            if false {
                console::log_2(&"p*v = ".into(), &Float32Array::from(mvp_flat.as_slice()));
                let origin = cgmath::vec4(0.0, 0.0, 0.0, 1.0);
                let xyzw = mvp * origin;
                console::log_2(
                    &"xyzw = ".into(),
                    &Float32Array::from(AsRef::<[f32; 4]>::as_ref(&xyzw).as_slice()),
                );

                let xyz = xyzw.div_element_wise(xyzw[3]).truncate();
                let xyz: &[f32; 3] = xyz.as_ref();
                console::log_4(
                    &"w=".into(),
                    &xyzw[3].into(),
                    &"origin transformed = ".into(),
                    &Float32Array::from(xyz.as_slice()),
                );
            }

            self.flat_shader
                .draw(gl, 0, 3, &self.triangle_vertices, mvp_flat);
        }
    }
}

fn projection_view_for(xr_view: &XrView) -> cgmath::Matrix4<f32> {
    let p = xr_view.projection_matrix();
    // console::log_2(&"proj= ".into(), &Float32Array::from(p.as_slice()));
    let view = xr_view.transform();
    // console::log_2(&"view= ".into(), &view);
    let vm = to_mat4(&view.matrix()).inverse_transform().unwrap();
    let pm = to_mat4(&p);
    pm * vm
}

pub fn to_mat4(src: &[f32]) -> cgmath::Matrix4<f32> {
    if src.len() == 16 {
        let mut rval = cgmath::Matrix4::from_scale(1.0);
        let x: &mut [f32; 16] = rval.as_mut();
        x.copy_from_slice(src);
        rval
    } else {
        panic!("matrix {}", src.len());
    }
}

//

pub struct AppInner {
    session: Option<XrSession>,
    gl: WebGl2RenderingContext,
    viewer_ref_space: Option<XrReferenceSpace>,
    draw_logic: DrawLogic,
}

impl AppInner {
    fn request_xr_session(xr: XrSystem, app: Rc<RefCell<AppInner>>) -> Result<Promise, JsValue> {
        if app.borrow().session.is_some() {
            Ok(Promise::resolve(&JsValue::from("Session already exists")))
        } else {
            log!("Y2 {}", xr.is_undefined());
            if xr.is_undefined() {
                return Err(JsValue::from("navigator.xr undefined"));
            }
            let session_mode = XrSessionMode::ImmersiveVr;
            log!("Y3");
            log!("{xr:?} {session_mode:?}");
            let session_supported_promise = xr.is_session_supported(session_mode);

            log!("Y4");
            // Note: &self is on the stack so we can't use it in a future (which will
            // run after the &self reference is out or scope). Clone ref to the parts
            // of self we'll need, then move them into the Future
            // See https://github.com/rustwasm/wasm-bindgen/issues/1858#issuecomment-552095511

            log!("Y2");

            let future_ = async move {
                let supports_session =
                    wasm_bindgen_futures::JsFuture::from(session_supported_promise).await;
                let supports_session = supports_session.unwrap();
                if supports_session == false {
                    log!("XR session not supported");
                    return Ok(JsValue::from("XR session not supported"));
                }

                let xr_session_promise = xr.request_session(session_mode);
                let xr_session = wasm_bindgen_futures::JsFuture::from(xr_session_promise).await;
                let xr_session: XrSession = xr_session?.into();

                log!("gl {:?}", &app.borrow_mut().gl);

                {
                    let gl = app.borrow().gl.clone();
                    JsFuture::from(gl.make_xr_compatible()).await?;
                }
                let xr_gl_layer = if true {
                    debug_new_layer(&xr_session, &app.borrow().gl)
                } else {
                    XrWebGlLayer::new_with_web_gl2_rendering_context(&xr_session, &app.borrow().gl)?
                };

                log!("layer created");
                let render_state_init = XrRenderStateInit::new();
                render_state_init.set_base_layer(Some(&xr_gl_layer));
                xr_session.update_render_state_with_state(&render_state_init);

                console::log_3(
                    &"space request ".into(),
                    &xr_session,
                    &XrReferenceSpaceType::Local.into(),
                );
                let world_ref_space =
                    xr_session.request_reference_space(XrReferenceSpaceType::Local);
                let world_ref_space = JsFuture::from(world_ref_space).await?;
                let world_ref_space = XrReferenceSpace::from(world_ref_space);

                console::log_2(&"space ".into(), &world_ref_space);

                let mut app1 = app.borrow_mut();
                log!("borrowed");
                app1.session = Some(xr_session);

                app1.viewer_ref_space = Some(world_ref_space);
                drop(app1);
                log!("unborrowed");

                request_animation_frame(
                    animation_callback(app.clone()).borrow().as_ref().unwrap(),
                    &app.borrow(),
                );

                Ok(JsValue::from("Session set"))
            };
            Ok(future_to_promise(future_))
        }
    }
}

//

#[wasm_bindgen]
pub struct XrApp {
    inner: Rc<RefCell<AppInner>>,
}

#[wasm_bindgen]
impl XrApp {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new() -> XrApp {
        set_panic_hook();

        let xr_mode = true;
        let tmp = create_webgl_context(xr_mode);
        if let Err(val) = &tmp {
            log!("{val:?}");
        }
        web_sys::console::log_1(tmp.as_ref().unwrap());
        let gl = tmp.unwrap();

        let draw_logic = DrawLogic::new(&gl).unwrap();
        let rval = XrApp {
            inner: Rc::new(RefCell::new(AppInner {
                session: None,
                gl,
                viewer_ref_space: None,
                draw_logic,
            })),
        };
        let _ = rval.attach_button();
        rval
    }

    fn attach_button(&self) -> Result<JsValue, JsValue> {
        let document = helper::document()?;
        let button = document
            .get_element_by_id("button")
            .ok_or_else(|| JsValue::from("no element button"))?;
        let inner = self.inner.clone();
        // let kludge = JsValue::from(kludge);
        let closure: Closure<dyn Fn() -> Result<Promise, JsValue>> =
            Closure::wrap(Box::new(move || {
                log!("click");
                // let kludge = XrApp::from(kludge);
                log!("session {}", inner.borrow().session.is_some());
                Self::js_request_xr(inner.clone())
            }));
        button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget(); // leak memory?
        Ok("ok".into())
    }

    fn js_request_xr(app: Rc<RefCell<AppInner>>) -> Result<Promise, JsValue> {
        log!("requseting xr");
        match Self::js_request_xr_(app) {
            Err(e) => {
                helper::append_to_document(&format!("boom {e:?}"))?;
                Err(e)
            }
            Ok(x) => Ok(x),
        }
    }

    fn js_request_xr_(app: Rc<RefCell<AppInner>>) -> Result<Promise, JsValue> {
        helper::append_to_document(" requesting xr ")?;
        let navigator = helper::window()?.navigator();
        AppInner::request_xr_session(navigator.xr(), app)
    }

    pub fn start(&self) {
        log!("XrApp::start()");
        request_animation_frame(
            animation_callback(self.inner.clone())
                .borrow()
                .as_ref()
                .unwrap(),
            &self.inner.borrow(),
        );
    }

    fn draw(timestamp: f64, xr_frame: &XrFrame, inner_app: &AppInner) {
        log!("draw");
        let draw_logic = &inner_app.draw_logic;
        //let inner_app = inner.borrow();
        match inner_app.session.as_ref() {
            Some(session) => {
                draw_logic.draw_xr(
                    &inner_app.gl,
                    timestamp,
                    xr_frame,
                    inner_app.viewer_ref_space.as_ref().unwrap(),
                    session,
                );
            }
            None => {
                draw_logic.draw(&inner_app.gl);
            }
        }
    }
}

//

pub fn animation_callback(
    app: Rc<RefCell<AppInner>>,
) -> Rc<RefCell<Option<Closure<dyn FnMut(f64, XrFrame)>>>> {
    let cell = Rc::new(RefCell::new(None));
    let f = cell.clone();
    *cell.borrow_mut() = Some(Closure::new(move |timestamp: f64, xr_frame: XrFrame| {
        //log!("debug");
        //draw_logic.draw(gl.as_ref());
        XrApp::draw(timestamp, &xr_frame, &app.borrow());
        request_animation_frame(f.borrow().as_ref().unwrap(), &app.borrow());
    }));
    cell
}

pub fn request_animation_frame(callback: &Closure<dyn FnMut(f64, XrFrame)>, app: &AppInner) {
    match app.session.as_ref() {
        None => {
            // let callback = Rc::new(RefCell::new(callback));
            // let f = callback.clone();

            window()
                .unwrap()
                .request_animation_frame(
                    //f.borrow().as_ref().unchecked_ref()
                    callback.as_ref().unchecked_ref(),
                )
                .unwrap();
        }
        Some(session) => {
            request_animation_frame_xr(session, callback);
        }
    }
}
