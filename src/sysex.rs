use nih_plug::{nih_log, prelude::SysExMessage};
use serde::{Deserialize, Serialize};

use crate::lyrics::Lyric;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct SysExLyric {
    raw: [u8; 6],
    short: bool,
}

impl SysExMessage for SysExLyric {
    type Buffer = [u8; 6];

    fn from_buffer(buffer: &[u8]) -> Option<Self> {
        let processed_sysex: [u8; 6];
        let four_byte: bool;
        nih_log!("Lyric SysEx: {:x?}", buffer);
        if buffer.len() == 4 {
            processed_sysex = [buffer[0], buffer[1], buffer[2], buffer[3], 0x00, buffer[3]];
            four_byte = true;
        } else {
            processed_sysex = buffer.try_into().unwrap();
            four_byte = false;
        }
        Option::Some(Self {
            raw: processed_sysex,
            short: four_byte,
        })
    }

    fn to_buffer(self) -> (Self::Buffer, usize) {
        (self.raw, 6)
    }
}

impl Default for SysExLyric {
    fn default() -> Self {
        SysExLyric::from_buffer([0xF0, 0x30, 0x42, 0xF7].as_ref()).unwrap()
    }
}

impl SysExLyric {
    pub fn is_lyric(&self) -> bool {
        nih_log!("Lyric: {:x?}, {}", self.raw, self.raw.len());
        if self.raw.len() != 6 {
            return false;
        }
        if self.raw[0] == 0xff && self.raw.last().unwrap_or(&0u8).clone() == 0x05 {
            nih_log!("Lyric using lyric event: {:x?}", self.raw);
            return true;
        }
        if self.raw[0] == 0xff && self.raw.last().unwrap_or(&0u8).clone() == 0x01 {
            nih_log!("Lyric using text event: {:x?}", self.raw);
            return true;
        }
        if self.raw[0] == 0xf0 && self.raw.last().unwrap_or(&0u8).clone() == 0xf7 {
            nih_log!("Lyric using SysEx event: {:x?}", self.raw);
            return true;
        }
        false
    }
}

impl Lyric for SysExLyric {
    fn get_jpn_utf8(&mut self) -> String {
        let lyric: [u8; 4];
        if self.short {
            lyric = [self.raw[1], self.raw[2], 0, 0];
            nih_log!("Lyric short: {:x?}", lyric);
        } else {
            lyric = [self.raw[1], self.raw[2], self.raw[3], self.raw[4]];
            nih_log!("Lyric long: {:x?}", lyric);
        }
        let mut lyric_16: Vec<u16> = vec![];
        lyric_16.push(((lyric[0] as u16) << 8) | lyric[1] as u16);
        if !self.short {
            lyric_16.push(((lyric[2] as u16) << 8) | lyric[3] as u16);
        }
        String::from_utf16_lossy(&lyric_16).trim().to_string()
    }
    fn get_latin(&mut self) -> String {
        //TODO: implement conversion from jpn_utf8 to latin
        "".to_string()
    }
}
