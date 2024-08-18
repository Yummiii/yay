use gstreamer::{Element, ElementFactory};

pub fn make(name: &str) -> Element {
    ElementFactory::make(name).name(name).build().unwrap()
}
