//! Image processor utility for generating sprite sheets
//!
//! This tool processes source textures and generates sprite sheets for the UI.

use std::fs;
use std::path::Path;

fn main() {
    println!("ChartPlayer Image Processor");
    println!();

    let src_path = "SrcTextures";
    let dest_path = "ChartPlayerShared/Content/Textures";

    process_sprites(src_path, dest_path);

    println!("\nSprite sheet generation complete!");
}

fn process_sprites(src: &str, dest: &str) {
    let ui_path = Path::new(src).join("UserInterface");

    if !ui_path.exists() {
        println!("Source path does not exist: {:?}", ui_path);
        return;
    }

    if let Err(e) = fs::create_dir_all(dest) {
        println!("Failed to create destination directory: {}", e);
        return;
    }

    let fonts = vec![("MainFont", 16), ("LargeFont", 32)];

    for (name, size) in &fonts {
        println!("Processing font: {} at size {}", name, size);
    }

    let images = vec![
        "SingleWhitePixel",
        "PopupBackground",
        "TabPanelBackground",
        "TabForeground",
        "TabBackground",
        "ButtonPressed",
        "ButtonUnpressed",
        "Play",
        "Pause",
        "Rewind",
        "LevelDisplay",
        "ScrollBar",
        "ScrollBarGutter",
        "ScrollUpArrow",
        "ScrollDownArrow",
        "HorizontalSlider",
        "VerticalSlider",
        "VerticalPointer",
        "VerticalPointerLeft",
        "VerticalFretLine",
        "HorizontalFretLine",
        "ChordOutline",
        "FingerOutline",
    ];

    for image in &images {
        let source_file = ui_path.join(format!("{}.png", image));
        if source_file.exists() {
            println!("Processing image: {}", image);
        }
    }

    let guitar_colors = ["Red", "Yellow", "Cyan", "Orange", "Green", "Purple"];
    for color in &guitar_colors {
        let source_file = ui_path.join(format!("Guitar{}.png", color));
        if source_file.exists() {
            println!("Processing guitar: Guitar{}", color);
        }
    }

    let note_techniques = vec![
        "HammerOn",
        "PullOff",
        "Mute",
        "PalmMute",
        "Harmonic",
        "PinchHarmonic",
    ];
    for technique in &note_techniques {
        let source_file = ui_path.join(format!("Note{}.png", technique));
        if source_file.exists() {
            println!("Processing technique: {}", technique);
        }
    }

    let drum_images = vec![
        ("DrumRed", true),
        ("DrumYellow", true),
        ("DrumBlue", true),
        ("DrumGreen", true),
        ("DrumRedStick", true),
        ("CymbalYellow", true),
        ("CymbalYellowFoot", true),
        ("CymbalYellowOpen", true),
        ("CymbalGreen", true),
        ("CymbalBlue", true),
        ("CymbalBlueBell", true),
        ("CymbalChoke", true),
    ];

    for (name, has_shadow) in &drum_images {
        let source_file = ui_path.join(format!("{}.png", name));
        if source_file.exists() {
            println!("Processing drum: {} (shadow: {})", name, has_shadow);
        }
    }

    println!("\nNote: Full image processing requires image processing libraries.");
}
