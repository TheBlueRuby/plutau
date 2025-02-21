use crate::playing_sample::PlayingSample;
use editor_vizia::visualizer::VisualizerData;
use nih_plug_vizia::ViziaState;
use rubato::Resampler;
use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use rtrb;

use nih_plug::prelude::*;
mod editor_vizia;
mod playing_sample;

mod phoneme;
use phoneme::Phoneme;

/// A loaded sample stored as a vec of samples in the form:
/// [
///     [a, a, a, ...],
///     [b, b, b, ...],
/// ]
pub struct LoadedSample(Vec<Vec<f32>>);

#[derive(Clone)]
pub enum ThreadMessage {
    LoadSinger(PathBuf),
    RemoveSinger(PathBuf),
}

/// Main plugin struct
pub struct Plutau {
    pub params: Arc<PlutauParams>,
    pub playing_samples: Vec<PlayingSample>,
    pub sample_rate: f32,
    pub loaded_samples: HashMap<PathBuf, LoadedSample>,
    pub consumer: RefCell<Option<rtrb::Consumer<ThreadMessage>>>,
    pub visualizer: Arc<VisualizerData>,
    pub lyric: Phoneme,
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
            lyric: Phoneme::new(0, 0),
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

    #[id = "note"]
    pub note: IntParam,

    #[id = "min-velocity"]
    pub min_velocity: IntParam,
    #[id = "max-velocity"]
    pub max_velocity: IntParam,

    #[id = "min-volume"]
    pub min_volume: FloatParam,
    #[id = "max-volume"]
    pub max_volume: FloatParam,

    #[id = "vowel"]
    pub vowel: IntParam,
    #[id = "consonant"]
    pub consonant: IntParam,

}

impl Default for PlutauParams {
    fn default() -> Self {
        Self {
            editor_state: ViziaState::new(|| (400, 700)),
            sample_list: Mutex::new(vec![]),
            note: IntParam::new("Note", 40, IntRange::Linear { min: 0, max: 127 }),
            min_velocity: IntParam::new("Min velocity", 0, IntRange::Linear { min: 0, max: 127 }),
            max_velocity: IntParam::new("Max velocity", 127, IntRange::Linear { min: 0, max: 127 }),
            min_volume: FloatParam::new(
                "Min volume",
                util::db_to_gain(0.0),
                FloatRange::Linear { min: 0.0, max: 2.0 },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            max_volume: FloatParam::new(
                "Max volume",
                util::db_to_gain(0.0),
                FloatRange::Linear { min: 0.0, max: 2.0 },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            singer_dir: Mutex::new(String::from("")),
            vowel: IntParam::new("Vowel", 0, IntRange::Linear { min: 0, max: 4 }),
            consonant: IntParam::new("Consonant", 0, IntRange::Linear { min: 0, max: 14 }),
        }
    }
}

impl Plugin for Plutau {
    const NAME: &'static str = "Plutau";
    const VENDOR: &'static str = "avi!86";
    const URL: &'static str = "https://avi86.bandcamp.com";
    const EMAIL: &'static str = "info@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::Basic;

    type SysExMessage = ();
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
        let singer = Path::new(self.params.singer_dir.lock().unwrap().clone().as_str()).to_path_buf();

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
        self.process_messages();
        self.process_midi(context, buffer);

        let mut amplitude = 0.0;

        for playing_sample in &mut self.playing_samples {
            // attempt to get sample data
            if let Some(loaded_sample) = self.loaded_samples.get(&playing_sample.handle) {
                // channel_samples is [a, b, c]
                for channel_samples in buffer.iter_samples() {
                    // if sample isnt in the future
                    if playing_sample.position >= 0 {
                        for (channel_index, sample) in channel_samples.into_iter().enumerate() {
                            let s = loaded_sample
                                .0
                                .get(channel_index)
                                .unwrap_or(&vec![])
                                .get(playing_sample.position as usize)
                                .unwrap_or(&0.0)
                                * playing_sample.gain;
                            *sample += s;
                            amplitude += s.abs();
                        }
                    }
                    playing_sample.position += 1;
                }
            }
        }

        amplitude /= buffer.samples() as f32 * buffer.channels() as f32;
        self.visualizer.store(amplitude);

        // remove samples that are done playing
        self.playing_samples
            .retain(|e| match self.loaded_samples.get(&e.handle) {
                Some(sample) => e.position < sample.0[0].len() as isize,
                None => false,
            });

        ProcessStatus::Normal
    }
}

fn uninterleave(samples: Vec<f32>, channels: usize) -> LoadedSample {
    // input looks like:
    // [a, b, a, b, a, b, ...]
    //
    // output should be:
    // [
    //    [a, a, a, ...],
    //    [b, b, b, ...]
    // ]

    let mut new_samples = vec![Vec::with_capacity(samples.len() / channels); channels];

    for sample_chunk in samples.chunks(channels) {
        // sample_chunk is a chunk like [a, b]
        for (i, sample) in sample_chunk.into_iter().enumerate() {
            new_samples[i].push(sample.clone());
        }
    }

    LoadedSample(new_samples)
}

fn resample(samples: LoadedSample, sample_rate_in: f32, sample_rate_out: f32) -> LoadedSample {
    let samples = samples.0;
    let mut resampler = rubato::FftFixedIn::<f32>::new(
        sample_rate_in as usize,
        sample_rate_out as usize,
        samples[0].len(),
        8,
        samples.len(),
    )
    .unwrap();

    match resampler.process(&samples, None) {
        Ok(mut waves_out) => {
            // get the duration of leading silence introduced by FFT
            // https://github.com/HEnquist/rubato/blob/52cdc3eb8e2716f40bc9b444839bca067c310592/src/synchro.rs#L654
            let silence_len = resampler.output_delay();

            for channel in waves_out.iter_mut() {
                channel.drain(..silence_len);
                channel.shrink_to_fit();
            }

            LoadedSample(waves_out)
        }
        Err(_) => LoadedSample(vec![]),
    }
}

impl Plutau {
    fn velocity_to_gain(&self, velocity: u8) -> f32 {
        let min_vol = self.params.min_volume.value();
        let max_vol = self.params.max_volume.value();
        let min_vel = self.params.min_velocity.value() as u8;
        let max_vel = self.params.max_velocity.value() as u8;
        let diff_vol = max_vol - min_vol;
        let diff_vel = (max_vel - min_vel) as f32;
        // this is just mapping from the velocity range to volume range
        min_vol + diff_vol * (velocity - min_vel) as f32 / diff_vel
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
                    NoteEvent::NoteOn { note, velocity, .. }
                        if (velocity * 127.0) as u8 >= self.params.min_velocity.value() as u8
                            && (velocity * 127.0) as u8
                                <= self.params.max_velocity.value() as u8 =>
                    {
                        self.lyric = Phoneme::new(self.params.vowel.value() as u8, self.params.consonant.value() as u8);
                        nih_log!("playing note: {}", note);
                        let phoneme = self.params.singer_dir.lock().unwrap().clone()
                            + std::path::MAIN_SEPARATOR_STR
                            + self.lyric.get_chars().as_str()
                            + ".wav";
                        nih_log!("playing phoneme: {}", phoneme);
                        // None if no samples are loaded
                        if let Some((path, _sample_data)) = self
                            .loaded_samples
                            .get_key_value(Path::new(phoneme.as_str()))
                        {
                            let mut playing_sample = PlayingSample::new(
                                path.clone(),
                                self.velocity_to_gain((velocity * 127.0) as u8),
                            );

                            // start at correct position in buffer
                            playing_sample.position = -(event.timing() as isize);

                            self.playing_samples.push(playing_sample);
                        }
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
            for channel in samples.0.iter_mut() {
                for sample in channel.iter_mut() {
                    *sample *= 128.0
                }
            }

            // If sample is in mono, duplicate the channel
            if samples.0.len() == 1 {
                samples.0.push(samples.0[0].clone());
            } else {
                let sample_length = samples.0[0].len();
                if samples.0[1] == vec![0.0f32; sample_length] {
                    samples.0[1] = samples.0[0].clone();
                }
            }

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
        if fs::read_dir(path.clone()).is_err() {
            nih_log!("failed to load singer from {}", path.to_str().unwrap());
            return;
        }
        for wav_file in fs::read_dir(path.clone()).unwrap() {
            let wav_file = wav_file.unwrap();
            let wav_path = wav_file.path();
            if wav_path.extension().unwrap_or_default() == "wav" {
                self.load_sample(wav_path);
            }
        }
        *self.params.singer_dir.lock().unwrap() = path.clone().to_str().unwrap().to_string();
        nih_log!("loaded singer from {}", path.to_str().unwrap());
    }
    fn remove_singer(&mut self, _path: PathBuf)  {
        let keys: Vec<PathBuf> = self.params.sample_list.lock().unwrap().clone();
        for path in keys {
            self.remove_sample(path);
        }
        *self.params.singer_dir.lock().unwrap() = String::from("");
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
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Generator,
        Vst3SubCategory::Sampler,
        Vst3SubCategory::Instrument,
    ];
}

nih_export_clap!(Plutau);
nih_export_vst3!(Plutau);
