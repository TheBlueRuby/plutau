#[derive(Debug, Clone, Copy)]
pub struct Phoneme {
    pub vowel: u8, // a,i,u,e,o
    pub consonant: u8, // none, k,s,t,n,h,m,y,r,w
}

impl Phoneme {
    pub fn new(vowel: u8, consonant: u8) -> Self {
        Self {
            vowel,
            consonant,
        }
    }

    pub fn get_chars(&self) -> String {
        let vowels = ['a', 'i', 'u', 'e', 'o'];
        let consonants = ["","k", "s", "t", "n", "h", "m", "y", "r", "w", "g", "z", "d", "b", "p"];
        format!("{}{}", consonants[self.consonant as usize], vowels[self.vowel as usize])
    }

    pub fn get_jpn_utf8(&self) -> String {
        let characters: Vec<Vec<&str>> = vec![
            vec!["あ", "い", "う", "え", "お"],
            vec!["か", "き", "く", "け", "こ"],
            vec!["さ", "し", "す", "せ", "そ"],
            vec!["た", "ち", "つ", "て", "と"],
            vec!["な", "に", "ぬ", "ね", "の"],
            vec!["は", "ひ", "ふ", "へ", "ほ"],
            vec!["ま", "み", "む", "め", "も"],
            vec!["や", "", "ゆ", "", "よ"],
            vec!["ら", "り", "る", "れ", "ろ"],
            vec!["わ", "", "", "", "を"],
            vec!["が", "ぎ", "ぐ", "げ", "ご"],
            vec!["ざ", "じ", "ず", "ぜ", "ぞ"],
            vec!["だ", "ぢ", "づ", "で", "ど"],
            vec!["ば", "び", "ぶ", "べ", "ぼ"],
            vec!["ぱ", "ぴ", "ぷ", "ぺ", "ぽ"],
        ];
        characters[self.consonant as usize][self.vowel as usize].to_string()
    }

    pub fn get_jpn_jis(&self) -> Vec<u8> {
        // Shift-JIS is not unicode-compatible so character bytes need to be stored instead
        let characters: Vec<Vec<&[u8]>> = vec![
            vec![b"\x82\xa0", b"\x82\xa2", b"\x82\xa4", b"\x82\xa6", b"\x82\xa8"], // あ, い, う, え, お
            vec![b"\x82\xa9", b"\x82\xab", b"\x82\xad", b"\x82\xaf", b"\x82\xb1"], // か, き, く, け, こ
            vec![b"\x82\xb3", b"\x82\xb5", b"\x82\xb7", b"\x82\xb9", b"\x82\xbb"], // さ, し, す, せ, そ
            vec![b"\x82\xbd", b"\x82\xbf", b"\x82\xc2", b"\x82\xc4", b"\x82\xc6"], // た, ち, つ, て, と
            vec![b"\x82\xc8", b"\x82\xc9", b"\x82\xca", b"\x82\xcb", b"\x82\xcc"], // な, に, ぬ, ね, の
            vec![b"\x82\xcd", b"\x82\xcf", b"\x82\xd1", b"\x82\xd3", b"\x82\xd5"], // は, ひ, ふ, へ, ほ
            vec![b"\x82\xd7", b"\x82\xd9", b"\x82\xdb", b"\x82\xdd", b"\x82\xdf"], // ま, み, む, め, も
            vec![b"\x82\xe0", b"", b"\x82\xe2", b"", b"\x82\xe4"], // や, "", ゆ, "", よ
            vec![b"\x82\xe6", b"\x82\xe8", b"\x82\xe9", b"\x82\xea", b"\x82\xeb"], // ら, り, る, れ, ろ
            vec![b"\x82\xed", b"", b"", b"", b"\x82\xf0"], // わ, "", "", "", を
            vec![b"\x82\xf2", b"\x82\xf4", b"\x82\xf6", b"\x82\xf8", b"\x82\xfa"], // が, ぎ, ぐ, げ, ご
            vec![b"\x82\xfc", b"\x82\xfd", b"\x82\xfe", b"\x82\xff", b"\x83\x00"], // ざ, じ, ず, ぜ, ぞ
            vec![b"\x83\x01", b"\x83\x02", b"\x83\x03", b"\x83\x04", b"\x83\x05"], // だ, ぢ, づ, で, ど
            vec![b"\x83\x06", b"\x83\x07", b"\x83\x08", b"\x83\x09", b"\x83\x0a"], // ば, び, ぶ, べ, ぼ
            vec![b"\x83\x0b", b"\x83\x0c", b"\x83\x0d", b"\x83\x0e", b"\x83\x0f"], // ぱ, ぴ, ぷ, ぺ, ぽ
        ];
        characters[self.consonant as usize][self.vowel as usize].to_vec()
    }
}