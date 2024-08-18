use std::io::{stdout, Write};

use gstreamer::{
    prelude::{Displayable, ElementExt, ElementExtManual},
    query::Seeking,
    ClockTime, Element, ElementFactory, Format, Message, MessageView, SeekFlags, State,
};

struct Data {
    playbin: Element,
    playing: bool,
    terminate: bool,
    seek_enabled: bool,
    seek_done: bool,
    duration: Option<ClockTime>,
}

fn main() {
    gstreamer::init().unwrap();

    let playbin = ElementFactory::make("playbin")
        .name("playbin")
        .property("uri", "file:///home/yummi/Downloads/a.mkv")
        .build()
        .unwrap();

    playbin.set_state(State::Playing).unwrap();

    let bus = playbin.bus().unwrap();
    let mut data = Data {
        playbin,
        playing: false,
        terminate: false,
        seek_enabled: false,
        seek_done: false,
        duration: ClockTime::NONE,
    };

    while !data.terminate {
        let msg = bus.timed_pop(100 * ClockTime::MSECOND);

        if let Some(msg) = msg {
            handle_message(&mut data, &msg);
        } else {
            if data.playing {
                let position = data.playbin.query_position::<ClockTime>().unwrap();

                if data.duration == ClockTime::NONE {
                    data.duration = data.playbin.query_duration();
                }

                print!("\rPosition {} / {}", position, data.duration.display());
                stdout().flush().unwrap();

                if data.seek_enabled && data.seek_done && position > 10 * ClockTime::SECOND {
                    println!("\nReached 10s, performing seek...");
                    data.playbin
                        .seek_simple(
                            SeekFlags::FLUSH | SeekFlags::KEY_UNIT,
                            30 * ClockTime::SECOND,
                        )
                        .expect("Failed to seek.");
                    data.seek_done = true;
                }
            }
        }
    }

    data.playbin.set_state(State::Null).unwrap();
}

fn handle_message(data: &mut Data, msg: &Message) {
    match msg.view() {
        MessageView::DurationChanged(_) => {
            data.duration = ClockTime::NONE;
        }
        MessageView::StateChanged(state) => {
            if state.src().map(|s| s == &data.playbin).unwrap_or(false) {
                let new_state = state.current();
                let old_state = state.old();

                println!("Pipeline state changed from {old_state:?} to {new_state:?}");

                data.playing = new_state == State::Playing;
                if data.playing {
                    let mut seeking = Seeking::new(Format::Time);
                    if data.playbin.query(&mut seeking) {
                        let (seekable, start, end) = seeking.result();
                        data.seek_enabled = seekable;
                        if seekable {
                            println!("Seeking is ENABLED from {start} to {end}")
                        } else {
                            println!("Seeking is DISABLED for this stream.")
                        }
                    } else {
                        eprintln!("Seeking query failed.")
                    }
                }
            }
        }
        MessageView::Eos(_) => {
            data.terminate = true;
        }
        MessageView::Error(err) => {
            data.terminate = true;
        }
        _ => (),
    }
}
