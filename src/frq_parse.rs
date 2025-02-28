use std::path::PathBuf;

pub fn get_avg_frq(path: PathBuf) -> f32 {
    std::println!("path: {:?}", path);
    let bytes: Vec<u8> = std::fs::read(path).unwrap_or(vec![0; 20]);
    let avg_frq_bytes: [u8; 8] = bytes[12..20].try_into().unwrap();
    let avg_frq = f64::from_le_bytes(avg_frq_bytes);
    avg_frq as f32
}

pub fn get_frq_amp(path: PathBuf) -> (Vec<f32>, Vec<f32>) {
    std::println!("path: {:?}", path);
    let mut bytes: Vec<u8> = std::fs::read(path).unwrap_or(vec![0; 40]);
    bytes.drain(0..36);
    let num_chunks = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
    std::println!("num_chunks: {:?}", num_chunks);

    let mut frqs: Vec<f32> = Vec::new();
    let mut amps: Vec<f32> = Vec::new();

    for _ in 0..num_chunks {
        let frq_bytes: [u8; 8] = bytes[0..8].try_into().unwrap();
        let frq = f64::from_le_bytes(frq_bytes);
        frqs.push(frq as f32);
        std::println!("frq: {:?}", frq);

        let amp_bytes = bytes[8..16].try_into().unwrap();
        let amp = f64::from_le_bytes(amp_bytes);
        amps.push(amp as f32);
        std::println!("amp: {:?}", amp);

        bytes.drain(0..16);
    }

    (frqs, amps)
}

pub fn get_frq(path: PathBuf) -> Vec<f32> {
    let (frqs, _) = get_frq_amp(path);
    frqs
}

pub fn get_amp(path: PathBuf) -> Vec<f32> {
    let (_, amps) = get_frq_amp(path);
    amps
}
