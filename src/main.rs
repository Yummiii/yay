use gstreamer::{
    prelude::{ElementExt, GObjectExtManualGst, GstBinExtManual, GstObjectExt, PadExt},
    Bin, ClockTime, DebugGraphDetails, Element, ElementFactory, MessageView, Pipeline, State,
};
use gtk4::prelude::{Cast, ObjectExt};

fn main() {
    gstreamer::init().unwrap();

    let source = ElementFactory::make("uridecodebin")
        .name("source")
        .build()
        .unwrap();

    let convert = ElementFactory::make("audioconvert")
        .name("convert")
        .build()
        .unwrap();
    let resample = ElementFactory::make("audioresample")
        .name("resample")
        .build()
        .unwrap();
    let sink = ElementFactory::make("autoaudiosink")
        .name("sink")
        .build()
        .unwrap();
    let vconvert = ElementFactory::make("videoconvert")
        .name("vconvert")
        .build()
        .unwrap();
    let vsink = ElementFactory::make("autovideosink")
        .name("vsink")
        .build()
        .unwrap();
    let vertigo = ElementFactory::make("warptv")
        .name("vertigo")
        .build()
        .unwrap();

    let pipeline = Pipeline::with_name("test-pipeline");

    pipeline
        .add_many([&source, &convert, &resample, &sink, &vconvert, &vertigo, &vsink])
        .unwrap();
    Element::link_many([&convert, &resample, &sink]).unwrap();
    Element::link_many([&vconvert, &vertigo, &vsink]).unwrap();

    source.set_property("uri", "file:/home/yummi/Downloads/a.mkv");

    source.connect_pad_added(move |src, src_pad| {
        println!("Received new pad {} from {}", src_pad.name(), src.name());

        let new_pad_caps = src_pad.current_caps().unwrap();
        let new_pad_struct = new_pad_caps.structure(0).unwrap();
        let new_pad_type = new_pad_struct.name();

        let sink_pad = match new_pad_type.to_string().as_ref() {
            "audio/x-raw" => convert.static_pad("sink"),
            "video/x-raw" => vconvert.static_pad("sink"),
            _ => None,
        };

        if let Some(sink_pad) = sink_pad {
            if !sink_pad.is_linked() {
                let res = src_pad.link(&sink_pad);
                if res.is_err() {
                    println!("Type is {new_pad_type} but link failed.");
                } else {
                    println!("Link succeeded (type {new_pad_type}).");
                }
            }
        }
    });

    pipeline.set_state(State::Playing).unwrap();

    boilerplate(pipeline);
}

fn boilerplate(pipeline: Pipeline) {
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(ClockTime::NONE) {
        match msg.view() {
            MessageView::Error(err) => {
                eprintln!(
                    "Error received from element {:?} {}",
                    err.src().map(|s| s.path_string()),
                    err.error()
                );
                eprintln!("Debugging information: {:?}", err.debug());
                break;
            }
            MessageView::StateChanged(state_changed) => {
                if state_changed.src().map(|s| s == &pipeline).unwrap_or(false) {
                    println!(
                        "Pipeline state changed from {:?} to {:?}",
                        state_changed.old(),
                        state_changed.current()
                    );
                }
            }
            MessageView::Eos(..) => break,
            _ => (),
        }
    }

    pipeline
        .set_state(State::Null)
        .expect("Unable to set the pipeline to the `Null` state");
}
