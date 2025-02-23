#![windows_subsystem = "windows"]

use nih_plug::prelude::*;

use plutau::Plutau;

fn main() {
    nih_export_standalone::<Plutau>();
}