
/// Converts a MIDI note number to a frequency in Hz.
/// This version accepts a float for the MIDI note number.
pub fn midi_to_hz(note: f32) -> f32 {
    // The MIDI note number 69 corresponds to A4, which is 440 Hz.
    let a4_freq = 440.0;
    let a4_midi_note = 69.0;

    // Calculate the frequency using the formula for equal temperament.
    a4_freq * (2.0_f32).powf((note - a4_midi_note) / 12.0)
}
