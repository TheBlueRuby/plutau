use std::path::PathBuf;

fn get_avg_frq(path: PathBuf) -> f32 {
    let bytes: Vec<u8> = std::fs::read(path).unwrap();
    let avg_frq_bytes = &bytes[12..20];
    let avg_frq = f32::from_le_bytes(avg_frq_bytes.try_into().unwrap());
    avg_frq
}

fn get_midi_note_from_frq(frq: f32) -> u8 {
    let midi_note = (69.0 + 12.0 * (frq / 440.0).log2()).round() as u8;
    midi_note
}
