use std::path::PathBuf;

pub fn get_avg_frq(path: PathBuf) -> f32 {
    std::println!("path: {:?}", path);
    let bytes: Vec<u8> = std::fs::read(path).unwrap_or(vec![0; 20]);
    let avg_frq_bytes: [u8; 8] = bytes[12..20].try_into().unwrap();
    let avg_frq = f64::from_le_bytes(avg_frq_bytes);
    avg_frq as f32
}

pub fn get_midi_note_from_frq(frq: f32) -> u8 {
    let midi_note = (69.0 + 12.0 * (frq / 440.0).log2()).round() as u8;
    midi_note
}
