// Web-specific implementations
use js_sys::{Array, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

/// Create a blob URL from raw data for web platform
pub fn create_blob_url(data: &[u8], mime_type: &str) -> Option<String> {
    let uint8_array = Uint8Array::new_with_length(data.len() as u32);
    uint8_array.copy_from(data);

    let array = Array::new();
    array.push(&uint8_array.buffer().into());

    let mut blob_options = BlobPropertyBag::new();
    blob_options.type_(mime_type);

    Blob::new_with_u8_array_sequence_and_options(&array, &blob_options)
        .ok()
        .and_then(|blob| Url::create_object_url_with_blob(&blob).ok())
}

/// Trigger a download for web platform
pub fn trigger_download(url: &str, filename: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(anchor) = document.create_element("a") {
                if let Ok(anchor_element) = anchor.dyn_into::<HtmlAnchorElement>() {
                    anchor_element.set_href(url);
                    anchor_element.set_download(filename);

                    // Set display:none using setAttribute
                    let _ = anchor_element.set_attribute("style", "display: none");

                    if let Some(body) = document.body() {
                        let _ = body.append_child(&anchor_element);
                        anchor_element.click();
                        let _ = body.remove_child(&anchor_element);
                    }
                }
            }
        }
    }
}
