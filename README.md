# Plutau

An UTAU plugin for your DAW  
Made with [nih-plug](https://github.com/robbert-vdh/nih-plug.git).  
Based on [nih-sampler](https://github.com/matidfk/nih-sampler).

## Features:
- Resampling with [TD-PSOLA](https://codeberg.org/PieterPenninckx/tdpsola)
- Loads UTF-8 CV Utauloids

## Usage

- Click "Add Singer" and browse to your Utau's folder (open the folder with your oto.ini and select)
- Input melody with a MIDI sequence (Monophonic, notes played while first is held will change pitch)
- Choose phonemes by automating Vowel and Consonant parameters

## TODO:
- UI improvements
- Shift-JIS to UTF-8 for oto parsing
- Declicking
- Smooth pitch bend & tuning

## Building

Build with:

```cargo xtask bundle plutau --release```

## License
All code is licensed under the [GPLv3](https://www.gnu.org/licenses/gpl-3.0.txt) license.

Font [Audiowide](https://fonts.google.com/specimen/Audiowide) is licensed under the [Open Font License v1.1](https://openfontlicense.org/open-font-license-official-text/)
