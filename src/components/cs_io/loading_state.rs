#![allow(unused)]

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::prelude::*;
        use web_sys;
        use crossbeam_channel::Receiver;
        use web_sys::HtmlElement;
    }
}


pub fn set_loading_finish() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let win = web_sys::window().unwrap();
            let document = win.document().unwrap();
            let element = document.get_element_by_id("loading_spinner").unwrap();
            let element = element.dyn_into::<HtmlElement>().unwrap();
            element.set_attribute("tag", "true").unwrap();
            element.click();
        }
    }
}


pub fn set_loading(loading: Option<&str>) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let win = web_sys::window().unwrap();
            let document = win.document().unwrap();
            let element = document.get_element_by_id("loading_text").unwrap();
            let element = element.dyn_into::<HtmlElement>().unwrap();
            element.set_text_content(loading);
        }
    }
}