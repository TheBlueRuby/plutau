use std::path::PathBuf;

use crate::{phoneme::Phoneme, sysex::SysExLyric};

trait Lyric {
    fn get_jpn_utf8(&self) -> String;
    fn get_jpn_jis(&self) -> Vec<u8> {
        let utf8_lut: Vec<&str> = vec![
            "あ", "い", "う", "え", "お", "か", "き", "く", "け", "こ", "さ", "し", "す", "せ",
            "そ", "た", "ち", "つ", "て", "と", "な", "に", "ぬ", "ね", "の", "は", "ひ", "ふ",
            "へ", "ほ", "ま", "み", "む", "め", "も", "や", "", "ゆ", "", "よ", "ら", "り", "る",
            "れ", "ろ", "わ", "", "", "", "を", "が", "ぎ", "ぐ", "げ", "ご", "ざ", "じ", "ず",
            "ぜ", "ぞ", "だ", "ぢ", "づ", "で", "ど", "ば", "び", "ぶ", "べ", "ぼ", "ぱ", "ぴ",
            "ぷ", "ぺ", "ぽ",
        ];
        let jis_lut: Vec<&[u8]> = vec![
            b"\x82\xa0",
            b"\x82\xa2",
            b"\x82\xa4",
            b"\x82\xa6",
            b"\x82\xa8",
            // あ, い, う, え, お
            b"\x82\xa9",
            b"\x82\xab",
            b"\x82\xad",
            b"\x82\xaf",
            b"\x82\xb1",
            // か, き, く, け, こ
            b"\x82\xb3",
            b"\x82\xb5",
            b"\x82\xb7",
            b"\x82\xb9",
            b"\x82\xbb",
            // さ, し, す, せ, そ
            b"\x82\xbd",
            b"\x82\xbf",
            b"\x82\xc2",
            b"\x82\xc4",
            b"\x82\xc6",
            // た, ち, つ, て, と
            b"\x82\xc8",
            b"\x82\xc9",
            b"\x82\xca",
            b"\x82\xcb",
            b"\x82\xcc",
            // な, に, ぬ, ね, の
            b"\x82\xcd",
            b"\x82\xcf",
            b"\x82\xd1",
            b"\x82\xd3",
            b"\x82\xd5",
            // は, ひ, ふ, へ, ほ
            b"\x82\xd7",
            b"\x82\xd9",
            b"\x82\xdb",
            b"\x82\xdd",
            b"\x82\xdf",
            // ま, み, む, め, も
            b"\x82\xe0",
            b"",
            b"\x82\xe2",
            b"",
            b"\x82\xe4", // や, "", ゆ, "", よ
            b"\x82\xe6",
            b"\x82\xe8",
            b"\x82\xe9",
            b"\x82\xea",
            b"\x82\xeb",
            // ら, り, る, れ, ろ
            b"\x82\xed",
            b"",
            b"",
            b"",
            b"\x82\xf0", // わ, "", "", "", を
            b"\x82\xf2",
            b"\x82\xf4",
            b"\x82\xf6",
            b"\x82\xf8",
            b"\x82\xfa",
            // が, ぎ, ぐ, げ, ご
            b"\x82\xfc",
            b"\x82\xfd",
            b"\x82\xfe",
            b"\x82\xff",
            b"\x83\x00",
            // ざ, じ, ず, ぜ, ぞ
            b"\x83\x01",
            b"\x83\x02",
            b"\x83\x03",
            b"\x83\x04",
            b"\x83\x05",
            // だ, ぢ, づ, で, ど
            b"\x83\x06",
            b"\x83\x07",
            b"\x83\x08",
            b"\x83\x09",
            b"\x83\x0a",
            // ば, び, ぶ, べ, ぼ
            b"\x83\x0b",
            b"\x83\x0c",
            b"\x83\x0d",
            b"\x83\x0e",
            b"\x83\x0f",
            // ぱ, ぴ, ぷ, ぺ, ぽ
        ];
        let utf8 = self.get_jpn_utf8();
        //map indexes from utf8_lut to jis_lut
        let mut jis_vec: Vec<u8> = vec![];
        for c in utf8.chars() {
            if let Some(index) = utf8_lut.iter().position(|&x| x == c.to_string()) {
                if let Some(jis_bytes) = jis_lut.get(index) {
                    jis_vec.extend_from_slice(jis_bytes);
                }
            }
        }
        if jis_vec.is_empty() {
            return vec![0x00, 0x00];
        }
        jis_vec
    }
    fn get_latin(&self) -> String;
}

pub enum LyricSource {
    Param,
    File,
    SysEx,
}

pub struct LyricSettings {
    pub lyric_source: LyricSource,
    pub lyric_file: Option<FileLyric>,
    pub lyric_sysex: Option<SysExLyric>,
    pub lyric_param: Option<ParamLyric>,
}

pub struct FileLyric {
    pub path: PathBuf,
    pub lyric_vec: Vec<String>,
    pub index: usize,
}

pub struct ParamLyric {
    pub current: Phoneme,
}
