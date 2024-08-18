use gstreamer::{
    message::Application, prelude::ElementExt, ClockTime, Element, ElementFactory, Message, State,
    Structure,
};
use gtk4::{
    gdk::Paintable, glib::Value, prelude::{ApplicationExt, ApplicationExtManual, ObjectExt, ToValue}, Box, HeaderBar, ListBox, Orientation, Picture, SelectionMode, Video, Widget
};
use libadwaita::{
    prelude::{ActionRowExt, BoxExt, GtkWindowExt},
    ActionRow, ApplicationWindow,
};
use utils::make;

mod utils;

struct Data {
    playbin: Element,
    sink_widget: Option<Paintable>,
    state: State,
    duration: Option<ClockTime>,
}

fn main() {
    libadwaita::init().unwrap();
    gtk4::init().unwrap();
    let app = libadwaita::Application::builder()
        .application_id("com.sexo")
        .build();
    gstreamer::init().unwrap();

    let mut data = Data {
        playbin: make("playbin"),
        sink_widget: None,
        state: State::Null,
        duration: None,
    };

    let videosink = make("glsinkbin");
    let gtkglsink = make("gtk4paintablesink");

    videosink.set_property("sink", &gtkglsink);
    let aaaa = gtkglsink.property::<Paintable>("paintable");

    println!("{:?}", data.sink_widget);

    data.playbin
        .set_property("uri", "file:///home/yummi/Downloads/a.mkv");
    data.playbin.set_property("video-sink", &videosink);

    data.playbin.connect("video-tags-changed", false, tags_cb);
    data.playbin.connect("audio-tags-changed", false, tags_cb);
    data.playbin.connect("text-tags-changed", false, tags_cb);



    app.connect_activate(move |app| {
        let content = Box::new(Orientation::Vertical, 0);

        content.append(&HeaderBar::new());
        // let a = &data.sink_widget.unwrap();

        let a = Picture::new();
        a.set_paintable(Some(&aaaa));

        content.append(&a);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("First App")
            .default_width(350)
            .content(&content)
            .build();
        window.present();
    });

    let bus = data.playbin.bus().unwrap();

    bus.add_signal_watch();
    bus.connect("message::error", false, |a| {
        println!("erro");
        None
    });
    bus.connect("message::eos", false, |a| {
        println!("eos");
        None
    });
    bus.connect("message::state-changed", false, |a| {
        println!("a");
        None
    });
    bus.connect("message::application", false, |a| {
        println!("app");
        None
    });

    data.playbin.set_state(State::Playing).unwrap();
    app.run();
}


fn tags_cb<'a>(value: &'a [Value]) -> Option<Value> {
    if let [playbin, _] = value {
        let playbin = playbin.get::<Element>().unwrap();

        playbin
            .post_message(Application::new(Structure::new_empty("tags-changed")))
            .unwrap();
    }
    None
}
