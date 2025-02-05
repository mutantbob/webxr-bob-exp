#![cfg(web_sys_unstable_apis)]

#[macro_use]
mod utils;

use js_sys::{Date, Object, Promise, Reflect};
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
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

fn request_animation_frame(session: &XrSession, f: &Closure<dyn FnMut(f64, XrFrame)>) -> u32 {
    // This turns the Closure into a js_sys::Function
    // See https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#casting-a-closure-to-a-js_sysfunction
    session.request_animation_frame(f.as_ref().unchecked_ref())
}

pub fn create_webgl_context(xr_mode: bool) -> Result<WebGl2RenderingContext, JsValue> {
    fn jserr(msg: &str) -> JsValue {
        msg.into()
    }
    let canvas = web_sys::window()
        .ok_or_else(|| jserr("window"))?
        .document()
        .ok_or_else(|| jserr("document"))?
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

struct DrawLogic {}

impl DrawLogic {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, gl: &WebGl2RenderingContext) {
        //log!("compute b");
        let x = Date::now();
        //log!("x {x:?}");
        let b = (x % 2000.0) as f32 / 2000.0;
        // log!("draw {b}");
        gl.clear_color(0.0, 1.0, b, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    }
}

#[wasm_bindgen]
pub struct XrApp {
    session: Rc<RefCell<Option<XrSession>>>,
    gl: Rc<WebGl2RenderingContext>,
}

#[wasm_bindgen]
impl XrApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> XrApp {
        set_panic_hook();

        let session = Rc::new(RefCell::new(None));

        let xr_mode = true;
        log!("D");
        let tmp = create_webgl_context(xr_mode);
        if let Err(val) = &tmp {
            log!("{val:?}");
        }
        let gl = Rc::new(tmp.unwrap());

        log!("E");
        XrApp { session, gl }
    }

    pub fn init(&self) -> Promise {
        log!("Starting WebXR...");
        let navigator: web_sys::Navigator = match web_sys::window() {
            None => {
                log!("missing window");
                panic!("beets");
            }
            Some(w) => w.navigator(),
        };
        log!("Y1");
        let xr = navigator.xr();

        if xr.is_undefined() {
            log!("undefined return from navigator.xr()");
            Promise::resolve(&"no XR".into())
        } else {
            log!("Y2 {}", xr.is_undefined());
            let session_mode = XrSessionMode::Inline;
            log!("Y3");
            log!("{xr:?} {session_mode:?}");
            let session_supported_promise = xr.is_session_supported(session_mode);

            log!("Y4");
            // Note: &self is on the stack so we can't use it in a future (which will
            // run after the &self reference is out or scope). Clone ref to the parts
            // of self we'll need, then move them into the Future
            // See https://github.com/rustwasm/wasm-bindgen/issues/1858#issuecomment-552095511
            let session = self.session.clone();
            let gl = self.gl.clone();

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
                let xr_session: XrSession = xr_session.unwrap().into();

                let xr_gl_layer =
                    XrWebGlLayer::new_with_web_gl2_rendering_context(&xr_session, &gl)?;
                let render_state_init = XrRenderStateInit::new();
                render_state_init.set_base_layer(Some(&xr_gl_layer));
                xr_session.update_render_state_with_state(&render_state_init);

                let mut session = session.borrow_mut();
                session.replace(xr_session);

                Ok(JsValue::from("Session set"))
            };

            log!("Y3");
            future_to_promise(future_)
        }
    }

    pub fn start(&self) {
        log!("start()");
        let draw_logic = DrawLogic::new();

        let gl = self.gl.clone();

        log!("A");

        let session: &Option<XrSession> = &self.session.borrow();
        if let Some(sess) = session {
            let f = Rc::new(RefCell::new(None));
            let g = f.clone();

            *g.borrow_mut() = Some(Closure::new(move |_time: f64, frame: XrFrame| {
                Self::frame_draw_xr(f.borrow_mut().deref_mut(), &draw_logic, &gl, frame);
            }));

            log!("request animation frame");
            request_animation_frame(sess, g.borrow().as_ref().unwrap());
        } else {
            let f = Rc::new(RefCell::new(None));
            let g = f.clone();

            *g.borrow_mut() = Some(Closure::new(move |_time: f64| {
                Self::frame_draw_2d(f.borrow_mut().deref_mut(), &draw_logic, &gl);
            }));

            let window = window();
            if let Some(window) = window {
                if window.is_undefined() {
                    log!("undefined window");
                } else {
                    let callback = g.borrow();
                    let callback = callback.as_ref().unwrap();
                    if let Err(e) =
                        window.request_animation_frame(callback.as_ref().unchecked_ref())
                    {
                        log!("failed to request_animation_frame {e:?}");
                    }
                }
            } else {
                log!("no window");
            }
        }

        log!("meh");
    }

    fn frame_draw_xr(
        callback_me: &mut Option<Closure<dyn FnMut(f64, XrFrame)>>,
        draw_logic: &DrawLogic,
        gl: &Rc<WebGl2RenderingContext>,
        frame: XrFrame,
    ) -> bool {
        log!("Frame rendering...");
        log!("frame = {frame:?}");

        let sess: XrSession = frame.session();

        draw_logic.draw(&gl);

        // Schedule ourself for another requestAnimationFrame callback.
        // TODO: WebXR Samples call this at top of request_animation_frame - should this be moved?
        request_animation_frame(&sess, callback_me.as_ref().unwrap());
        false
    }

    fn frame_draw_2d(
        callback_me: &mut Option<Closure<dyn FnMut(f64)>>,
        draw_logic: &DrawLogic,
        gl: &Rc<WebGl2RenderingContext>,
    ) -> bool {
        draw_logic.draw(&gl);

        // Schedule ourself for another requestAnimationFrame callback.
        // TODO: WebXR Samples call this at top of request_animation_frame - should this be moved?
        // let callback = callback_me.borrow();
        let callback = callback_me.as_ref().unwrap();
        window()
            .unwrap()
            .request_animation_frame(callback.as_ref().unchecked_ref())
            .unwrap();
        false
    }
}
