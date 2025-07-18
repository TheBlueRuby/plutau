use crate::playing_sample::PlayingSample;
use editor_vizia::visualizer::VisualizerData;
use nih_plug_vizia::ViziaState;
use playing_sample::PlayingState;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    vec,
};
use tdpsola::{AlternatingHann, Speed, TdpsolaAnalysis, TdpsolaSynthesis};

use nih_plug::prelude::*;
mod editor_vizia;
mod playing_sample;

mod sample_util;
use sample_util::*;

mod frq_parse;
use frq_parse::*;

mod oto;
use oto::*;

mod lyrics;
use lyrics::*;

mod sysex;
use sysex::*;

mod phoneme;
use phoneme::*;

mod midi;
use midi::*;

/// A loaded sample stored as a vec of samples in the form:
/// [
///     [a, a, a, ...],
///     [b, b, b, ...],
/// ]
pub struct LoadedSample {
    samples: Vec<Vec<f32>>,
    frequency: f32,
}

#[derive(Clone)]
pub enum ThreadMessage {
    LoadSinger(PathBuf),
    RemoveSinger(PathBuf),
    LoadLyric(PathBuf),
    SetLyricSource(i32),
}

/// Main plugin struct
pub struct Plutau {
    pub params: Arc<PlutauParams>,
    pub playing_samples: Vec<PlayingSample>,
    pub sample_rate: f32,
    pub loaded_samples: HashMap<PathBuf, LoadedSample>,
    pub consumer: RefCell<Option<rtrb::Consumer<ThreadMessage>>>,
    pub visualizer: Arc<VisualizerData>,
    pub sample_frequency: f32,
    pub midi_frequency: f32,
    pub pitch_bend: f32,
    pub note: u8,
    pub lyric: String,
}

impl Default for Plutau {
    fn default() -> Self {
        Self {
            params: Arc::new(Default::default()),
            playing_samples: vec![],
            loaded_samples: HashMap::with_capacity(64),
            consumer: RefCell::new(None),
            sample_rate: 44100.0,
            visualizer: Arc::new(VisualizerData::new()),
            sample_frequency: 440.0,
            midi_frequency: 440.0,
            pitch_bend: 0.0,
            note: 0,
            lyric: String::new(),
        }
    }
}

/// Plugin parameters struct
#[derive(Params)]
pub struct PlutauParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    #[persist = "sample-list"]
    sample_list: Mutex<Vec<PathBuf>>,
    #[persist = "singer-dir"]
    pub singer_dir: Mutex<String>,
    #[persist = "oto"]
    pub oto: Mutex<Oto>,
    #[persist = "lyric-settings"]
    pub lyric_settings: Arc<Mutex<LyricSettings>>,

    pub singer: Arc<Mutex<String>>,
    pub cur_sample: Arc<Mutex<String>>,
    pub lyrics: Arc<Mutex<String>>,

    #[id = "vowel"]
    pub vowel: IntParam,

    #[id = "consonant"]
    pub consonant: IntParam,

    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "instant-cutoff"]
    pub instant_cutoff: BoolParam,
    #[id = "bend-range"]
    pub bend_range: FloatParam,
    #[id = "crossfade-length"]
    pub crossfade_length: IntParam,
    #[id = "crossfade-on"]
    pub crossfade_on: BoolParam,
}

impl Default for PlutauParams {
    fn default() -> Self {
        Self {
            editor_state: ViziaState::new(|| (800, 700)),
            sample_list: Mutex::new(vec![]),
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Linear { min: 0.0, max: 4.0 },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            instant_cutoff: BoolParam::new("Instant Cutoff", true),
            lyric_settings: Arc::new(Mutex::new(LyricSettings::new())),
            singer_dir: Mutex::new(String::from("")),
            singer: Arc::new(Mutex::new(String::from("None"))),
            cur_sample: Arc::new(Mutex::new(String::from(""))),
            lyrics: Arc::new(Mutex::new(String::from(""))),
            vowel: IntParam::new("Vowel", 0, IntRange::Linear { min: 0, max: 4 }),
            consonant: IntParam::new("Consonant", 0, IntRange::Linear { min: 0, max: 14 }),
            oto: Mutex::new(Oto::new(String::from(""))),
            bend_range: FloatParam::new(
                "Bend Range",
                2.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 24.0,
                },
            )
            .with_unit(" semitones")
            .with_step_size(1.0),
            crossfade_length: IntParam::new(
                "Crossfade Length",
                100,
                IntRange::Linear { min: 0, max: 1000 },
            )
            .with_unit(" samples"),
            crossfade_on: BoolParam::new("Crossfade", true),
        }
    }
}

impl Plugin for Plutau {
    const NAME: &'static str = "Plutau";
    const VENDOR: &'static str = "avi!86";
    const URL: &'static str = "https://avi86.bandcamp.com";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;
    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::MidiCCs;

    type SysExMessage = SysExLyric;
    type BackgroundTask = ();

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let (producer, consumer) = rtrb::RingBuffer::new(10);
        self.consumer.replace(Some(consumer));

        editor_vizia::create(
            self.params.clone(),
            self.params.singer.clone(),
            self.params.cur_sample.clone(),
            self.params.lyrics.clone(),
            self.params.editor_state.clone(),
            Arc::new(Mutex::new(producer)),
            Arc::clone(&self.visualizer),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        nih_log!("changed sample rate to {}", buffer_config.sample_rate);

        self.sample_rate = buffer_config.sample_rate;
        let singer =
            Path::new(self.params.singer_dir.lock().unwrap().clone().as_str()).to_path_buf();

        self.remove_singer(singer.clone());
        self.load_singer(singer.clone());

        return true;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // clear buffers to prevent buzzing sound on daws which dont
        for buf in buffer.as_slice().iter_mut() {
            for s in buf.iter_mut() {
                *s = 0.0;
            }
        }

        self.process_messages();
        self.process_midi(context, buffer);

        let mut amplitude = 0.0;

        if !context.transport().playing {
            // reset lyric index, may cause desyncs if not playing from start of song
            self.params.lyric_settings.lock().unwrap().lyric_file.index = 0;
        }

        for playing_sample in &mut self.playing_samples {
            // attempt to get sample data
            if let Some(loaded_sample) = self.loaded_samples.get(&playing_sample.handle) {
                let source_frequency = self.sample_frequency as f32;
                let target_frequency = self.midi_frequency as f32;
                //--------------------------------

                let sampling_frequency = self.sample_rate as f32;

                let source_wavelength = sampling_frequency as f32 / source_frequency as f32;
                let target_wavelength = sampling_frequency as f32 / target_frequency as f32;
                let speed = Speed::from_f32(1.0);

                // Left
                let mut l_alternating_hann = AlternatingHann::new(source_wavelength);
                let mut l_analysis = TdpsolaAnalysis::new(&l_alternating_hann);

                let l_padding_length = source_wavelength as usize + 1;
                for _ in 0..l_padding_length {
                    l_analysis.push_sample(0.0, &mut l_alternating_hann);
                }
                let l_in: Vec<f32> = loaded_sample.samples[0].clone();

                for sample in l_in.iter() {
                    l_analysis.push_sample(*sample, &mut l_alternating_hann);
                }

                let mut l_synthesis = TdpsolaSynthesis::new(speed, target_wavelength);
                let mut l_out: Vec<f32> = Vec::new();
                for output_sample in l_synthesis.iter(&l_analysis).skip(l_padding_length) {
                    l_out.push(output_sample);
                }

                // Right
                let mut r_alternating_hann = AlternatingHann::new(source_wavelength);
                let mut r_analysis = TdpsolaAnalysis::new(&r_alternating_hann);

                let r_padding_length = source_wavelength as usize + 1;
                for _ in 0..r_padding_length {
                    r_analysis.push_sample(0.0, &mut r_alternating_hann);
                }
                let r_in: Vec<f32> = loaded_sample.samples[1].clone();

                for sample in r_in.iter() {
                    r_analysis.push_sample(*sample, &mut r_alternating_hann);
                }

                let mut r_synthesis = TdpsolaSynthesis::new(speed, target_wavelength);
                let mut r_out: Vec<f32> = Vec::new();
                for output_sample in r_synthesis.iter(&r_analysis).skip(r_padding_length) {
                    r_out.push(output_sample);
                }

                let shifted_sample: LoadedSample = LoadedSample {
                    samples: vec![l_out.clone(), r_out.clone()],
                    frequency: target_frequency,
                };
                // channel_samples is [a, b, c]
                for channel_samples in buffer.iter_samples() {
                    // if sample isnt in the future
                    if playing_sample.position >= 0 {
                        for (channel_index, sample) in channel_samples.into_iter().enumerate() {
                            let s = shifted_sample
                                .samples
                                .get(channel_index)
                                .unwrap_or(&vec![])
                                .get(playing_sample.position as usize)
                                .unwrap_or(&0.0)
                                * playing_sample.gain;

                            if self.params.crossfade_on.value() {
                                if playing_sample.ignore_fade {
                                    *sample += s;
                                    amplitude += s.abs();
                                } else {
                                    nih_log!(
                                        "pos: {}, start: {}, end: {}, crossfade start: {}",
                                        playing_sample.position,
                                        playing_sample.vowel_start,
                                        playing_sample.vowel_end,
                                        playing_sample.vowel_end
                                            - self.params.crossfade_length.value() as u32
                                    );
                                    // If crossfade has started, average samples from current point and from the loop start with offset
                                    if playing_sample.position
                                        >= (playing_sample.vowel_end
                                            - self.params.crossfade_length.value() as u32)
                                            as isize
                                    {
                                        let offset = playing_sample.position
                                            - (playing_sample.vowel_end
                                                - self.params.crossfade_length.value() as u32)
                                                as isize;
                                        nih_log!(
                                            "crossfade offset: {}, new sample pos: {}",
                                            offset,
                                            playing_sample.vowel_start as isize + offset as isize
                                        );
                                        let s2 = shifted_sample
                                            .samples
                                            .get(channel_index)
                                            .unwrap_or(&vec![])
                                            .get(
                                                playing_sample.vowel_start as usize
                                                    + offset as usize,
                                            )
                                            .unwrap_or(&0.0)
                                            * playing_sample.gain;
                                        nih_log!("s: {}, s2: {}", s, s2);
                                        let ratio = offset as f32
                                            / self.params.crossfade_length.value() as f32;
                                        nih_log!(
                                            "with ratio {}: s: {}, s2: {}",
                                            ratio,
                                            s * (1.0 - ratio),
                                            s2 * ratio
                                        );
                                        *sample +=
                                            s * (1.0 - ratio) + s2 * ratio * playing_sample.gain;
                                        amplitude += (s.abs() * (1.0 - ratio))
                                            + (s2.abs() * ratio) * playing_sample.gain;
                                    } else {
                                        *sample += s;
                                        amplitude += s.abs();
                                    }
                                }
                            } else {
                                *sample += s;
                                amplitude += s.abs();
                            }
                        }
                    }
                    playing_sample.position += 1;

                    match playing_sample.state {
                        PlayingState::ATTACK => {
                            if playing_sample.position >= playing_sample.vowel_start as isize {
                                playing_sample.state = PlayingState::SUSTAIN;
                            }
                        }
                        PlayingState::SUSTAIN => {
                            if playing_sample.position > playing_sample.vowel_end as isize {
                                playing_sample.position = (playing_sample.vowel_start
                                    + self.params.crossfade_length.value() as u32)
                                    as isize;
                            }
                            if playing_sample.position
                                > (playing_sample.vowel_start
                                    + self.params.crossfade_length.value() as u32)
                                    as isize
                            {
                                playing_sample.ignore_fade = false;
                            }
                        }
                        PlayingState::RELEASE => {
                            playing_sample.ignore_fade = true;
                            playing_sample.position = playing_sample.vowel_end as isize;
                            playing_sample.state = PlayingState::DONE;
                        }
                        _ => {}
                    }
                }
            }
        }

        amplitude /= buffer.samples() as f32 * buffer.channels() as f32;
        self.visualizer.store(amplitude);

        // remove samples that are done playing
        self.playing_samples
            .retain(|e| match self.loaded_samples.get(&e.handle) {
                Some(sample) => e.position < sample.samples[0].len() as isize,
                None => false,
            });
        if self.params.instant_cutoff.value() {
            self.playing_samples
                .retain(|e| match self.loaded_samples.get(&e.handle) {
                    Some(_sample) => e.state != PlayingState::DONE,
                    None => false,
                });
        } else {
            for playing_sample in &mut self.playing_samples {
                if playing_sample.state == PlayingState::DONE
                    && playing_sample.position < playing_sample.vowel_end as isize
                {
                    playing_sample.position = playing_sample.vowel_end as isize;
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl Plutau {
    fn velocity_to_gain(&self, velocity: u8) -> f32 {
        let max_vol = self.params.gain.value();
        // this is just mapping from the velocity range to volume range
        max_vol * (velocity as f32 / 127.0)
    }

    fn process_messages(&mut self) {
        let mut consumer = self.consumer.take();
        if let Some(consumer) = &mut consumer {
            while let Ok(message) = consumer.pop() {
                match message {
                    ThreadMessage::LoadSinger(path) => {
                        self.remove_singer(path.clone());
                        self.load_singer(path.clone());
                    }
                    ThreadMessage::RemoveSinger(path) => {
                        self.remove_singer(path.clone());
                    }
                    ThreadMessage::LoadLyric(path) => {
                        self.load_lyric(path.clone());
                    }
                    ThreadMessage::SetLyricSource(source) => {
                        // map int to enum
                        let source = match source {
                            0 => LyricSource::Param,
                            1 => LyricSource::File,
                            2 => LyricSource::SysEx,
                            _ => LyricSource::Param,
                        };
                        self.params
                            .lyric_settings
                            .lock()
                            .unwrap()
                            .set_lyric_source(source.clone());
                        nih_log!("Set lyric source to {:?}", source);
                    }
                }
            }
        }

        self.consumer.replace(consumer);
    }

    fn process_midi(&mut self, context: &mut impl ProcessContext<Self>, buffer: &mut Buffer) {
        let mut next_event = context.next_event();

        for (sample_id, _channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        self.note = note;
                        self.midi_frequency = midi_to_hz(note as f32 + self.pitch_bend);
                        if !(self.playing_samples.is_empty()) {
                            if !(self.playing_samples[0].state == PlayingState::DONE
                                || self.playing_samples[0].state == PlayingState::RELEASE)
                            {
                                return;
                            }
                        }
                        nih_log!("playing note: {}", note);

                        // update lyric if not using sysex
                        self.params
                            .lyric_settings
                            .lock()
                            .unwrap()
                            .lyric_param
                            .current = Phoneme::new(
                            self.params.vowel.value() as u8,
                            self.params.consonant.value() as u8,
                        );

                        nih_log!(
                            "source: {:?}",
                            self.params.lyric_settings.lock().unwrap().lyric_source
                        );

                        self.lyric = self.params.lyric_settings.lock().unwrap().get_jpn_utf8();

                        // phoneme will be the path to the phoneme wav file
                        let phoneme = format!(
                            "{}{}{}.wav",
                            self.params.singer_dir.lock().unwrap().clone(),
                            std::path::MAIN_SEPARATOR_STR,
                            self.lyric.clone()
                        );
                        nih_log!("playing phoneme: {}", phoneme);
                        *self.params.cur_sample.lock().unwrap() = phoneme.clone();
                        // None if no samples are loaded
                        if let Some((path, sample_data)) =
                            self.loaded_samples.get_key_value(Path::new(&phoneme))
                        {
                            self.sample_frequency = sample_data.frequency;
                            let offset = (self
                                .params
                                .oto
                                .lock()
                                .unwrap()
                                .get_entry(self.lyric.clone() + ".wav")
                                .unwrap()
                                .offset as f32
                                / 1000.0)
                                * self.sample_rate;

                            nih_log!("sample length in samples: {}", sample_data.samples[0].len());

                            let mut playing_sample = PlayingSample::new(
                                path.clone(),
                                self.velocity_to_gain((velocity * 127.0) as u8),
                            );

                            playing_sample.vowel_start =
                                (((self
                                    .params
                                    .oto
                                    .lock()
                                    .unwrap()
                                    .get_entry(self.lyric.clone() + ".wav")
                                    .unwrap()
                                    .consonant as f32
                                    / 1000.0)
                                    * self.sample_rate)
                                    + offset) as u32;
                            playing_sample.vowel_end = (sample_data.samples[0].len() as f32
                                - ((self
                                    .params
                                    .oto
                                    .lock()
                                    .unwrap()
                                    .get_entry(self.lyric.clone() + ".wav")
                                    .unwrap()
                                    .cutoff as f32
                                    / 1000.0)
                                    * self.sample_rate))
                                as u32;

                            // start at correct position in buffer
                            playing_sample.position = -(event.timing() as isize);

                            playing_sample.state = PlayingState::ATTACK;

                            self.playing_samples.push(playing_sample);
                        }
                    }
                    NoteEvent::NoteOff {
                        timing: _timing,
                        voice_id: _voice_id,
                        channel: _channel,
                        note: _note,
                        velocity: _velocity,
                    } => {
                        self.playing_samples
                            .iter_mut()
                            .for_each(|e| e.state = PlayingState::RELEASE);
                    }
                    NoteEvent::MidiSysEx {
                        timing: _timing,
                        message,
                        ..
                    } => {
                        if message.is_lyric() {
                            self.params.lyric_settings.lock().unwrap().lyric_sysex = message;
                            *self.params.cur_sample.lock().unwrap() = format!(
                                "{}{}{}.wav",
                                self.params.singer_dir.lock().unwrap().clone(),
                                std::path::MAIN_SEPARATOR_STR,
                                self.lyric
                            );
                            nih_log!(
                                "Received lyric: {}",
                                self.params
                                    .lyric_settings
                                    .lock()
                                    .unwrap()
                                    .lyric_sysex
                                    .get_jpn_utf8()
                            );
                        } else {
                            nih_log!("Received SysEx message: {:?}", message);
                        }
                    }
                    NoteEvent::MidiPitchBend {
                        timing: _,
                        channel: _,
                        value,
                    } => {
                        self.pitch_bend = (value - 0.5) * 2.0 * self.params.bend_range.value();
                        self.midi_frequency = midi_to_hz(self.note as f32 + self.pitch_bend);
                    }
                    _ => (),
                }
                next_event = context.next_event();
            }
        }
    }

    /// Loads a sample at the given filepath, overwriting any sample loaded with the given path
    fn load_sample(&mut self, path: PathBuf) {
        // wav only for now
        let reader = hound::WavReader::open(&path);
        if let Ok(mut reader) = reader {
            let spec = reader.spec();
            let sample_rate = spec.sample_rate as f32;
            let channels = spec.channels as usize;

            let interleaved_samples = match spec.sample_format {
                hound::SampleFormat::Int => reader
                    .samples::<i32>()
                    .map(|s| (s.unwrap_or_default() as f32 * 256.0) / i32::MAX as f32)
                    .collect::<Vec<f32>>(),
                hound::SampleFormat::Float => reader
                    .samples::<f32>()
                    .map(|s| s.unwrap_or_default())
                    .collect::<Vec<f32>>(),
            };

            let mut samples = uninterleave(interleaved_samples, channels);

            // resample if needed
            if sample_rate != self.sample_rate {
                samples = resample(samples, sample_rate, self.sample_rate);
            }

            // Amplify to audible levels (-6.6dB)
            for channel in samples.samples.iter_mut() {
                for sample in channel.iter_mut() {
                    *sample *= 128.0
                }
            }

            // If sample is in mono, duplicate the channel
            if samples.samples.len() == 1 {
                samples.samples.push(samples.samples[0].clone());
            } else {
                let sample_length = samples.samples[0].len();
                if samples.samples[1] == vec![0.0f32; sample_length] {
                    samples.samples[1] = samples.samples[0].clone();
                }
            }

            let sample_frq = get_avg_frq(
                Path::new(
                    str::replace(path.clone().to_str().unwrap(), ".wav", "_wav.frq").as_str(),
                )
                .to_path_buf(),
            );
            samples.frequency = sample_frq;

            self.loaded_samples.insert(path.clone(), samples);
        }

        if !self.params.sample_list.lock().unwrap().contains(&path) {
            self.params.sample_list.lock().unwrap().push(path);
        }
    }

    fn remove_sample(&mut self, path: PathBuf) {
        let mut sample_list = self.params.sample_list.lock().unwrap();
        if let Some(index) = sample_list.iter().position(|e| e == &path) {
            sample_list.remove(index);
        }
        self.loaded_samples.remove(&path);
    }

    fn load_singer(&mut self, path: PathBuf) {
        self.remove_singer(path.clone());
        let oto_path =
            path.clone().to_str().unwrap().to_owned() + std::path::MAIN_SEPARATOR_STR + "oto.ini";
        if fs::read(oto_path.clone()).is_err() {
            nih_log!("failed to load singer from {}", oto_path.clone());
            return;
        }

        let mut oto = Oto::new(oto_path.clone());
        oto.load();
        *self.params.oto.lock().unwrap() = oto.clone();

        oto.contents.iter().for_each(|entry| {
            let path = path.clone().join(unsafe {
                std::ffi::OsString::from_encoded_bytes_unchecked(entry.clone().file)
            });
            self.load_sample(Path::new(path.as_os_str()).to_path_buf());
        });

        *self.params.singer_dir.lock().unwrap() = path.clone().to_str().unwrap().to_string();
        let singer_name = path
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string()
            .split(std::path::MAIN_SEPARATOR)
            .last()
            .unwrap()
            .to_string();
        nih_log!(
            "loaded singer {} from {}",
            singer_name,
            path.to_str().unwrap()
        );
        *self.params.singer.lock().unwrap() = path.to_str().unwrap().to_string();
    }
    fn remove_singer(&mut self, _path: PathBuf) {
        let keys: Vec<PathBuf> = self.params.sample_list.lock().unwrap().clone();
        for path in keys {
            self.remove_sample(path);
        }
        *self.params.singer_dir.lock().unwrap() = String::from("");
        *self.params.oto.lock().unwrap() = Oto::new(String::from(""));
        *self.params.singer.lock().unwrap() = String::from("None");
    }

    fn load_lyric(&mut self, path: PathBuf) {
        if let Ok(contents) = fs::read_to_string(&path) {
            *self.params.lyrics.lock().unwrap() = contents;
            self.params.lyric_settings.lock().unwrap().lyric_file = FileLyric::new(path);
        }
    }
}

impl ClapPlugin for Plutau {
    const CLAP_ID: &'static str = "com.avi86.plutau";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("An UTAU plugin for your DAW");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for Plutau {
    const VST3_CLASS_ID: [u8; 16] = *b"Avi86UtauPlugin1";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Generator, Vst3SubCategory::Instrument];
}

nih_export_clap!(Plutau);
nih_export_vst3!(Plutau);
