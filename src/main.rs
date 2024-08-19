use gstreamer::{
    prelude::{ElementExt, GstBinExtManual, PadExt},
    Element, Pipeline, State,
};
use gtk4::{
    gdk::Paintable,
    prelude::{ApplicationExt, ApplicationExtManual, BoxExt, GtkWindowExt, ObjectExt},
    Box, HeaderBar, Orientation, Picture,
};
use libadwaita::ApplicationWindow;
use utils::make;

mod utils;

fn main() {
    libadwaita::init().unwrap();
    let app = libadwaita::Application::builder()
        .application_id("com.sexo")
        .build();
    gstreamer::init().unwrap();

    let source = make("uridecodebin");
    source.set_property("uri", "https://cdn.discordapp.com/attachments/493396462030422026/1275184464955179060/Bad_Apple_-_Full_Version_wvideo_Lyrics_in_Romaji_Translation_in_English_9lNZ_Rnr7Jc.webm?ex=66c4f794&is=66c3a614&hm=7ce2861a025aa0a2220020ff6e3c33cfdb9c6d5e6cd5b5addf18f150a368d6c5&");

    let audio_convert = make("audioconvert");
    let audio_resample = make("audioresample");
    let audio_volume = make("volume");
    let audio_sink = make("autoaudiosink");

    let video_convert = make("videoconvert");
    let glsink = make("glsinkbin");
    let video_sink = make("gtk4paintablesink");
    let quarktv = make("rippletv");
    let efeito = make("edgetv");

    audio_volume.set_property("volume", 0.05);
    glsink.set_property("sink", &video_sink);

    let pipeline = Pipeline::with_name("test-pipeline");
    pipeline
        .add_many([
            &source,
            &audio_convert,
            &audio_resample,
            &audio_volume,
            &audio_sink,
            &video_convert,
            &quarktv,
            &glsink,
            &efeito
            // &video_sink,
        ])
        .unwrap();
    Element::link_many([&audio_convert, &audio_resample, &audio_volume, &audio_sink]).unwrap();
    Element::link_many([&video_convert, &efeito, &glsink]).unwrap();

    app.connect_activate(move |app| {
        let content = Box::new(Orientation::Vertical, 0);
        content.append(&HeaderBar::new());

        let video = Picture::new();
        video.set_paintable(Some(&video_sink.property::<Paintable>("paintable")));

        content.append(&video);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("First App")
            .default_width(350)
            .content(&content)
            .build();
        window.present();
    });

    source.connect_pad_added(move |src, pad| {
        let new_pad_caps = pad.current_caps().unwrap();
        let new_pad_struct = new_pad_caps.structure(0).unwrap();
        let new_pad_type = new_pad_struct.name();

        let sink_pad = match new_pad_type.to_string().as_ref() {
            "audio/x-raw" => audio_convert.static_pad("sink"),
            "video/x-raw" => video_convert.static_pad("sink"),
            _ => None,
        };

        if let Some(sink_pad) = sink_pad {
            if !sink_pad.is_linked() {
                let res = pad.link(&sink_pad);
                if res.is_err() {
                    println!("Type is {new_pad_type} but link failed.");
                    println!("{:?}", res.err());
                } else {
                    println!("Link succeeded (type {new_pad_type}).");
                }
            }
        }
    });

    pipeline.set_state(State::Playing).unwrap();
    app.run();
}
