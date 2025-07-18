use crate::LoadedSample;
use rubato::Resampler;

pub fn uninterleave(samples: Vec<f32>, channels: usize) -> LoadedSample {
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
            new_samples[i].push(*sample);
        }
    }

    LoadedSample {
        samples: new_samples,
        frequency: DEFAULT_FREQUENCY,
    }
}

pub fn resample(samples: LoadedSample, sample_rate_in: f32, sample_rate_out: f32) -> LoadedSample {
    let sample_data = samples.samples;
    let mut resampler = rubato::FftFixedIn::<f32>::new(
        sample_rate_in as usize,
        sample_rate_out as usize,
        sample_data[0].len(),
        8,
        sample_data.len(),
    )
    .unwrap();

    match resampler.process(&sample_data, None) {
        Ok(mut waves_out) => {
            // get the duration of leading silence introduced by FFT
            // https://github.com/HEnquist/rubato/blob/52cdc3eb8e2716f40bc9b444839bca067c310592/src/synchro.rs#L654
            let silence_len = resampler.output_delay();

            for channel in waves_out.iter_mut() {
                channel.drain(..silence_len);
                channel.shrink_to_fit();
            }

            LoadedSample {
                samples: waves_out,
                frequency: samples.frequency,
            }
        }
        Err(_) => LoadedSample {
            samples: vec![],
            frequency: 440.0,
        },
    }
}