#![allow(unused)]
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use nih_plug_vizia::{create_vizia_editor, vizia::image::Pixels, ViziaState, ViziaTheming};

use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;

use crate::{PlutauParams, ThreadMessage};

use super::visualizer::{Visualizer, VisualizerData};

#[derive(Lens)]
struct Data {
    params: Arc<PlutauParams>,
    singer_dir: Arc<Mutex<String>>,
    cur_sample: Arc<Mutex<String>>,
    lyrics: Arc<Mutex<String>>,
    producer: Arc<Mutex<rtrb::Producer<ThreadMessage>>>,
    debug: String,
    visualizer: Arc<VisualizerData>,
}

#[derive(Clone)]
enum AppEvent {
    OpenSingerFilePicker,
    LoadSinger(PathBuf),
    RemoveSinger(PathBuf),
    OpenLyricFilePicker,
    LoadLyric(PathBuf),
    SetLyricSource(i32),
}

impl Model for Data {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::OpenSingerFilePicker => {
                cx.spawn(|cx_proxy| {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        cx_proxy.emit(AppEvent::LoadSinger(path));
                    }
                });
            }
            AppEvent::LoadSinger(path) => {
                self.debug = format!("loading: {path:?}");
                if let Err(e) = self
                    .producer
                    .lock()
                    .unwrap()
                    .push(ThreadMessage::LoadSinger(path.clone()))
                {
                    self.debug = e.to_string();
                }
            }
            AppEvent::RemoveSinger(path) => {
                self.debug = format!("removing: {path:?}");
                if let Err(e) = self
                    .producer
                    .lock()
                    .unwrap()
                    .push(ThreadMessage::RemoveSinger(path.clone()))
                {
                    self.debug = e.to_string();
                }
            }
            AppEvent::OpenLyricFilePicker => {
                cx.spawn(|cx_proxy| {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        cx_proxy.emit(AppEvent::LoadLyric(path));
                    }
                });
            }
            AppEvent::LoadLyric(path) => {
                self.debug = format!("loading: {path:?}");
                if let Err(e) = self
                    .producer
                    .lock()
                    .unwrap()
                    .push(ThreadMessage::LoadLyric(path.clone()))
                {
                    self.debug = e.to_string();
                }
            }
            AppEvent::SetLyricSource(source) => {
                self.debug = format!("setting lyric source: {source}");
                if let Err(e) = self
                    .producer
                    .lock()
                    .unwrap()
                    .push(ThreadMessage::SetLyricSource(*source))
                {
                    self.debug = e.to_string();
                }
            }
        });
    }
}

pub fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (700, 700))
}

pub fn create(
    params: Arc<PlutauParams>,
    singer: Arc<Mutex<String>>,
    sample: Arc<Mutex<String>>,
    lyric_list: Arc<Mutex<String>>,
    editor_state: Arc<ViziaState>,
    producer: Arc<Mutex<rtrb::Producer<ThreadMessage>>>,
    visualizer: Arc<VisualizerData>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        cx.add_theme(include_str!("theme.css"));
        cx.add_fonts_mem(&[include_bytes!("./Audiowide-Regular.ttf")]);

        Data {
            params: params.clone(),
            singer_dir: singer.clone(),
            cur_sample: sample.clone(),
            lyrics: lyric_list.clone(),
            producer: producer.clone(),
            debug: "nothing".into(),
            visualizer: visualizer.clone(),
        }
        .build(cx);

        ResizeHandle::new(cx);
        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, "Plutau").id("logo");
                Label::new(cx, env!("CARGO_PKG_VERSION")).id("version");
                Visualizer::new(cx, Data::visualizer).id("visualizer");
            })
            .class("top-bar");
            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    // Label::new(cx, Data::debug).overflow(Overflow::Hidden);
                    Label::new(cx, "Settings").id("title");

                    Label::new(cx, "Lyric Source").class("heading");
                    // Picker for lyric source
                    Dropdown::new(
                        cx,
                        |cx| {
                            Label::new(
                                cx,
                                Data::params.map(|params| {
                                    match params.lyric_settings.lock().unwrap().lyric_source {
                                        crate::lyrics::LyricSource::Param => {
                                            "Parameters".to_string()
                                        }
                                        crate::lyrics::LyricSource::File => "File".to_string(),
                                        crate::lyrics::LyricSource::SysEx => "SysEx".to_string(),
                                    }
                                }),
                            )
                            .color(Color::black())
                            .width(Stretch(1.0))
                        },
                        |cx| {
                            for i in 0..=2 {
                                Label::new(
                                    cx,
                                    match i {
                                        0 => "Parameters - Automate Vowel and Consonant",
                                        1 => "File - Load space-separated phonemes",
                                        2 => "SysEx - Unicode bytes as SysEx messages",
                                        _ => unreachable!(),
                                    },
                                )
                                .on_press(move |cx| {
                                    cx.emit(AppEvent::SetLyricSource(i));
                                    cx.emit(PopupEvent::Close); // close the popup
                                })
                                .background_color(Color::black())
                                .width(Stretch(1.0));
                            }
                        },
                    )
                    .width(Stretch(1.0))
                    .min_height(Pixels(24.0));

                    Element::new(cx).height(Pixels(8.0));

                    GenericUi::new(cx, Data::params).id("settings-container");
                })
                .class("main-body")
                .class("vert-list");

                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "Lyrics").class("heading");
                        Button::new(
                            cx,
                            |cx| {
                                cx.emit(AppEvent::OpenLyricFilePicker);
                            },
                            |cx| Label::new(cx, "Choose Lyrics File").class("add-file-text"),
                        )
                        .class("add-file-button");
                    })
                    .height(Auto)
                    .col_between(Stretch(1.0));

                    ScrollView::new(cx, 0.0, 0.0, true, false, |cx| {
                        Label::new(
                            cx,
                            Data::params.map(|params| {
                                params
                                    .lyric_settings
                                    .lock()
                                    .unwrap()
                                    .clone()
                                    .lyric_file
                                    .path
                                    .as_os_str()
                                    .to_str()
                                    .unwrap()
                                    .to_string()
                            }),
                        )
                        .class("text-container")
                        .min_width(Pixels(360.0));

                        Label::new(
                            cx,
                            Data::lyrics.map(|lyrics| lyrics.lock().unwrap().clone()),
                        )
                        .class("text-container")
                        .min_width(Pixels(360.0));
                    })
                    .max_height(Pixels(48.0))
                    .class("lyric-scrollview");

                    Label::new(cx, "Singer Directory").class("heading");
                    Label::new(
                        cx,
                        Data::singer_dir.map(|singer| singer.lock().unwrap().clone()),
                    )
                    .class("text-container");

                    Label::new(cx, "Current Sample").class("heading");
                    Label::new(
                        cx,
                        Data::cur_sample.map(|sample| sample.lock().unwrap().clone()),
                    )
                    .class("text-container");

                    HStack::new(cx, |cx| {
                        Label::new(cx, "Loaded Samples").class("heading");

                        Button::new(
                            cx,
                            |cx| cx.emit(AppEvent::OpenSingerFilePicker),
                            |cx| Label::new(cx, "Choose Singer").class("add-file-text"),
                        )
                        .class("add-file-button");
                    })
                    .height(Auto)
                    .col_between(Stretch(1.0));

                    ScrollView::new(cx, 0.0, 0.0, false, true, |cx| {
                        List::new(
                            cx,
                            Data::params.map(|params| params.sample_list.lock().unwrap().clone()),
                            |cx, index, item| {
                                HStack::new(cx, |cx| {
                                    Label::new(
                                        cx,
                                        &item
                                            .get(cx)
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                            .to_string(),
                                    );
                                    Label::new(cx, "Remove All").class("remove-label").on_press(
                                        move |cx| {
                                            cx.emit(AppEvent::RemoveSinger(item.get(cx).clone()))
                                        },
                                    );
                                })
                                .class("sample");
                            },
                        )
                        .class("vert-list")
                        .class("sample-list");
                    })
                    .class("sample-scrollview");
                })
                .class("main-body")
                .class("vert-list");
            });
        })
        .id("container");
    })
}
fn param_row<L, Params, P, FMap>(cx: &mut Context, label: &str, params: L, params_to_param: FMap)
where
    L: Lens<Target = Params> + Clone,
    Params: 'static,
    P: Param + 'static,
    FMap: Fn(&Params) -> &P + Copy + 'static,
{
    HStack::new(cx, |cx| {
        Label::new(cx, label).class("param-label");
        ParamSlider::new(cx, params, params_to_param);
    })
    .class("row");
}
