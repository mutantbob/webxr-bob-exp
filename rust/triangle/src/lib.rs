#![cfg(web_sys_unstable_apis)]

#[macro_use]
mod utils;

use js_sys::{Date, Object, Promise, Reflect};
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
        gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );
    }

    pub fn draw_xr(
        &self,
        gl: &WebGl2RenderingContext,
        _timestamp: f64,
        frame: XrFrame,
        viewer_ref_space: &XrReferenceSpace,
        session: &XrSession,
    ) {
        log!("get pose");
        let viewer_pose = frame.get_viewer_pose(viewer_ref_space);
        let Some(viewer_pose) = viewer_pose else {
            return;
        };
        let gl_layer = session.render_state().base_layer().unwrap();

        gl.bind_framebuffer(
            WebGl2RenderingContext::FRAMEBUFFER,
            gl_layer.framebuffer().as_ref(),
        );
        //log!("compute b");
        let x = Date::now();
        //log!("x {x:?}");
        let b = (x % 2000.0) as f32 / 2000.0;
        // log!("draw {b}");
        gl.clear_color(0.0, 1.0, b, 1.0);
        gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        for view in viewer_pose.views() {
            // console::log_2(&"view ".into(), &view);
            let view = &XrView::from(view);
            let viewport = gl_layer.get_viewport(view).unwrap();
            console::log_2(&"viewport ".into(), &viewport);
            gl.viewport(
                viewport.x(),
                viewport.y(),
                viewport.width(),
                viewport.height(),
            );
            self.draw_xr_single(gl, view);
        }
    }

    pub fn draw_xr_single(&self, gl: &WebGl2RenderingContext, _view: &XrView) {
        //self.draw(gl);
    }
}

pub struct AppInner {
    session: Option<XrSession>,
    gl: WebGl2RenderingContext,
    viewer_ref_space: Option<XrReferenceSpace>,
}

#[wasm_bindgen]
pub struct XrApp {
    inner: Rc<RefCell<AppInner>>,
}

#[wasm_bindgen]
impl XrApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> XrApp {
        set_panic_hook();

        // let session = Rc::new(RefCell::new(None));

        let xr_mode = true;
        log!("D");
        let tmp = create_webgl_context(xr_mode);
        if let Err(val) = &tmp {
            log!("{val:?}");
        }
        web_sys::console::log_1(&tmp.as_ref().unwrap());
        let gl = tmp.unwrap();

        let rval = XrApp {
            inner: Rc::new(RefCell::new(AppInner {
                session: None,
                gl,
                viewer_ref_space: None,
            })),
        };
        let _ = rval.attach_button();
        rval
    }

    pub fn init(&self) -> Promise {
        if true {
            return Promise::resolve(&"skipped".into());
        }
        log!("Starting WebXR...");
        let navigator: web_sys::Navigator = match web_sys::window() {
            None => {
                log!("missing window");
                panic!("beets");
            }
            Some(w) => w.navigator(),
        };
        let xr = navigator.xr();

        if xr.is_undefined() {
            log!("undefined return from navigator.xr()");
            log!(" session {}", self.inner.borrow().session.is_some());
            Promise::resolve(&"no XR".into())
        } else {
            Self::request_xr_session_inner(xr, self.inner.clone())
                .expect("failed to initialize webxr session")
        }
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

    pub fn get_gl(&self) -> JsValue {
        JsValue::from(&self.inner.borrow().gl)
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
        Self::request_xr_session_inner(navigator.xr(), app)
    }

    fn request_xr_session_inner(
        xr: XrSystem,
        app: Rc<RefCell<AppInner>>,
    ) -> Result<Promise, JsValue> {
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

                let inner = app.borrow_mut();
                let gl = &inner.gl;
                log!("gl {:?}", gl);

                JsFuture::from(gl.make_xr_compatible()).await?;
                let xr_gl_layer = if true {
                    debug_new_layer(&xr_session, &gl)
                } else {
                    XrWebGlLayer::new_with_web_gl2_rendering_context(&xr_session, &gl)?
                };
                drop(inner);

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

                request_animation_frame(animation_callback(app.clone()), app);

                Ok(JsValue::from("Session set"))
            };
            Ok(future_to_promise(future_))
        }
    }

    pub fn start(&self) {
        log!("start()");

        let gl = &self.inner.borrow().gl;

        web_sys::console::log_2(&"A ".into(), gl);

        request_animation_frame(animation_callback(self.inner.clone()), self.inner.clone());

        log!("session {}", self.inner.borrow().session.is_some());

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
        request_animation_frame_xr(&sess, callback_me.as_ref().unwrap());
        false
    }

    fn draw(timestamp: f64, xr_frame: XrFrame, inner: Rc<RefCell<AppInner>>) {
        log!("draw");
        let draw_logic = DrawLogic::new();
        let inner_app = inner.borrow();
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

trait FlexPainter {
    fn draw_2d(&self, gl: &Rc<WebGl2RenderingContext>);
    fn draw_xr(&self, gl: &Rc<WebGl2RenderingContext>, xr_frame: XrFrame);
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
        XrApp::draw(timestamp, xr_frame, app.clone());
        request_animation_frame(f.clone(), app.clone());
    }));
    cell
}

pub fn request_animation_frame(
    callback: Rc<RefCell<Option<Closure<dyn FnMut(f64, XrFrame)>>>>,
    app: Rc<RefCell<AppInner>>,
) {
    let callback = callback.borrow();
    let callback = callback.as_ref().unwrap();
    match app.borrow().session.as_ref() {
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
