use std::path::PathBuf;

pub struct PlayingSample {
    pub handle: PathBuf,
    pub position: isize,
    pub gain: f32,
    pub state: PlayingState,
    pub vowel_start: u32,
    pub vowel_end: u32,
    pub ignore_fade: bool
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayingState {
    ATTACK,
    SUSTAIN,
    RELEASE,
    DONE,
}

impl PlayingSample {
    pub fn new(handle: PathBuf, gain: f32) -> Self {
        Self {
            handle,
            position: 0,
            gain,
            state: PlayingState::ATTACK,
            vowel_start: 0,
            vowel_end: 0,
            ignore_fade: true,
        }
    }
}
