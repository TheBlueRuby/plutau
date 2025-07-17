# Plutau

An UTAU plugin for your DAW  
Made with [nih-plug](https://github.com/robbert-vdh/nih-plug.git).  
Based on [nih-sampler](https://github.com/matidfk/nih-sampler).

## Features:
- Resampling with [TD-PSOLA](https://codeberg.org/PieterPenninckx/tdpsola)
- Loads Utauloids (CV and UTF-8 Only)

## Installation
- Copy the `plutau.vst3` folder to your VST3 directory. (`C:\Program Files\Common Files\VST3` on Windows)
- If your DAW supports CLAP, copy `plutau.clap` to your CLAP directory (`C:\Program Files\Common Files\CLAP` on Windows)
- Refresh your DAW's plugin list
- Check the Instruments/Generators section of the plugins menu. The VST3 version may be under Samplers.
- If possible, use the CLAP version of the plugin as it is the one that has been tested more.

## Usage

- Click "Add Singer" and browse to your Utau's folder (the one that contains oto.ini)
- Input melody with a MIDI sequence (monophonic)
- Enter the lyrics using one of the following methods:
    - Enter the lyrics directly into the UI (space-separated phonemes)
    - Automate the lyric parameters
    - Use SysEx events to enter characters. Use the UTF-8 codepoints (e.g. 0x30 0x42 for あ).

## Troubleshooting

### My Utau samples look garbled in the UI (wrong characters or missing character points)
Your Utau might be using Shift-JIS encoding. OpenUtau can convert them to UTF-8 banks.
You will also need to convert the oto.ini to UTF-8, which VS Code can do with "reopen with encoding" and "save as encoding".

### New notes play the last phoneme, not the new one
Sometimes the phoneme can be updated after the note is registered but still within the same processing cycle.
You can move the automation point to just before the new note and it should work, however I am still trying to figure out a proper solution.

### My issue isn't listed here
Check the TODO section below, if the issue isn't mentioned there, open an issue on the repository's issues page.
Support on the project is welcome, so if you have a solution or suggestion, please let me know!

## TODO:
- All methods of entering phonemes
- UI improvements
- Shift-JIS to UTF-8 for oto parsing
- Pitch bend decrackling
- Preutterance (maybe use latency compensation?)
- Detect whether the bank uses Hiragana, Katakana or Latin alphabet and adjust accordingly
- Detect whether the bank is UTF-8 or Shift-JIS and translate if needed
- Better updating of phonemes

## Building

Build with:

```cargo xtask bundle plutau --release```

## License
All code is licensed under the [GPLv3](https://www.gnu.org/licenses/gpl-3.0.txt) license.

Font [Audiowide](https://fonts.google.com/specimen/Audiowide) is licensed under the [Open Font License v1.1](https://openfontlicense.org/open-font-license-official-text/)
