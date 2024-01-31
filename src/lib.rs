use gloo_timers::future::TimeoutFuture;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use tokio::sync::broadcast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

extern crate console_error_panic_hook;

macro_rules! console_log {
    ($($t:tt)*) =>
        (web_sys::console::log_1(
            &wasm_bindgen::JsValue::from_str(&format!($($t)*))))
}

#[wasm_bindgen]
extern "C" {
    pub type JsFunction;
    #[wasm_bindgen(method, structural, js_name = call)]
    pub fn call(this: &JsFunction, this_arg: &JsValue, data: &JsValue);
}

#[wasm_bindgen]
pub struct Dummy {
    cmd_sender: broadcast::Sender<u8>,
    //another: Another,
    stop: Arc<AtomicBool>,
    counter: Arc<Mutex<u8>>,
}

#[wasm_bindgen]
impl Dummy {
    #[wasm_bindgen(constructor)]
    pub async fn new(progress_cb: JsFunction) -> Result<Dummy, JsValue> {
        if progress_cb.is_undefined() {
            return Err(JsValue::from_str("progress callback is undefined"));
        }

        let cloned_cb = JsFunction {
            obj: progress_cb.clone(),
        };
        let (cmd_sender, _) = broadcast::channel::<u8>(u8::MAX.into());

        let mut receiver = cmd_sender.subscribe();

        let stop = Arc::new(AtomicBool::new(false));
        let stop_ref = stop.clone();

        spawn_local(async move {
            loop {
                if let Ok(val) = receiver.recv().await {
                    console_log!("received {}", val);
                    cloned_cb.call(&JsValue::NULL, &val.into());
                    if val == u8::MAX {
                        console_log!("exiting receiver loop");
                        break;
                    }
                }

                if stop_ref.load(Ordering::Relaxed) == true {
                    console_log!("receiver: stop detected, breaking out");
                    break;
                }
            }
        });

        return Ok(Dummy {
            cmd_sender: cmd_sender,
            stop: stop,
            counter: Arc::new(Mutex::new(0)),
        });
    }

    pub async fn work(&self) {
        // has to be async, because it calls some async stuff in the real world,
        // not sure if it has any influence
        let mut val: u8 = 0;
        let stop = self.stop.clone();
        loop {
            val += 1;

            // try to lock the counter and see if that helps: it does, but
            // only if we also change &mut self to &self
            let mut locked = self.counter.lock().unwrap();
            *locked = val;
            drop(locked);

            let _ = self.cmd_sender.send(val);
            if val == u8::MAX {
                console_log!("exiting worker loop");
                break;
            }
            if stop.load(Ordering::Relaxed) == true {
                console_log!("work: stop detected, breaking out");
                break;
            }

            TimeoutFuture::new(1000).await;
        }
    }

    pub fn abort_work(&self) {
        // does not need to do anything in order to cause the error
        console_log!("abort function triggered");
        self.stop.store(true, Ordering::Relaxed);
    }
}
