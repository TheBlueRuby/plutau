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
        format!("{}{}", vowels[self.vowel as usize], consonants[self.consonant as usize])
    }
}