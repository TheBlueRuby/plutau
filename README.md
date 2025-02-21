# Plutau

![Screenshot](./screenshot.png)

An UTAU plugin for your DAW
Made with [nih-plug](https://github.com/robbert-vdh/nih-plug.git).
Based on [nih-sampler](https://github.com/matidfk/nih-sampler).

Run with:

`cargo xtask bundle plutau --release`

Features:
- Automatically reload and resample all samples when sample rate changes
- Min and max volume, the volume is calculated by mapping velocity
- Deterministic sample picker

## Usage

Input melody with a MIDI sequence
Use MIDI CC 16 to select vowels (0-4 => a, i, u, e, o)
Use MIDI CC 17 to select consonants (0-14 => none, k, s, t, n, h, m, y, r, w, g, z, d, b, p)


## TODO:
- Implement voicebank loading
- Implement pitch shifting
- find better font
- add different channel config support
- add icons

All code is licensed under the [GPLv3](https://www.gnu.org/licenses/gpl-3.0.txt) license.
