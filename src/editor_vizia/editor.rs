#![allow(unused)]
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};

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
    producer: Arc<Mutex<rtrb::Producer<ThreadMessage>>>,
    debug: String,
    visualizer: Arc<VisualizerData>,
}

#[derive(Clone)]
enum AppEvent {
    OpenFilePicker,
    LoadSinger(PathBuf),
    RemoveSinger(PathBuf),
}

impl Model for Data {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::OpenFilePicker => {
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
            producer: producer.clone(),
            debug: "nothing".into(),
            visualizer: visualizer.clone(),
        }
        .build(cx);

        ResizeHandle::new(cx);
        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Label::new(cx, "Plutau").id("logo");
                Visualizer::new(cx, Data::visualizer).id("visualizer");
            })
            .class("top-bar");

            VStack::new(cx, |cx| {
                // Label::new(cx, Data::debug).overflow(Overflow::Hidden);
                Label::new(cx, "Settings").class("heading");
                GenericUi::new(cx, Data::params).id("settings-container");

                Label::new(cx, "Singer Directory").class("heading");
                Label::new(cx, Data::singer_dir.map(|singer| singer.lock().unwrap().clone())).id("singer-container");

                Label::new(cx, "Current Sample").class("heading");
                Label::new(cx, Data::cur_sample.map(|sample| sample.lock().unwrap().clone())).id("sample-container");

                HStack::new(cx, |cx| {
                    Label::new(cx, "Loaded Samples").class("heading");

                    Button::new(
                        cx,
                        |cx| cx.emit(AppEvent::OpenFilePicker),
                        |cx| Label::new(cx, "Choose Singer").id("add-sample-text"),
                    )
                    .id("add-sample-button");
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
                                    move |cx| cx.emit(AppEvent::RemoveSinger(item.get(cx).clone())),
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
