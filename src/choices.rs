// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! Each format's own choices, as its options dialog shows them and the
//! plugin's settings keep them: one object per format, `{"wav": {"depth":
//! "24", "rate": "48000"}, "mp3": {...}}`, each choice by its key. What
//! the settings don't name, or name wrongly, is the choice's default.

use oxiguitar_plugin::serde_json::{Map, Value};

/// The formats, in the order of the export dialog's list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// WAV: PCM, every program opens it.
    Wav,
    /// FLAC: lossless and about half as large.
    Flac,
    /// MP3: lossy, plays everywhere.
    Mp3,
    /// Opus in Ogg: lossy and small.
    Opus,
}

impl Kind {
    /// Every format.
    pub const ALL: [Kind; 4] = [Kind::Wav, Kind::Flac, Kind::Mp3, Kind::Opus];

    /// The exporter's id in the manifest, and its object in the settings.
    pub fn id(self) -> &'static str {
        match self {
            Kind::Wav => "wav",
            Kind::Flac => "flac",
            Kind::Mp3 => "mp3",
            Kind::Opus => "opus",
        }
    }

    /// The id of its options dialog in the manifest.
    pub fn options(self) -> &'static str {
        match self {
            Kind::Wav => "wav-options",
            Kind::Flac => "flac-options",
            Kind::Mp3 => "mp3-options",
            Kind::Opus => "opus-options",
        }
    }

    /// The format of the exporter `id`, or of the options dialog `id`.
    pub fn of(id: &str) -> Option<Kind> {
        Kind::ALL
            .into_iter()
            .find(|k| k.id() == id || k.options() == id)
    }

    /// Its choices, in the dialog's order.
    pub fn fields(self) -> &'static [Field] {
        match self {
            Kind::Wav => &[WAV_DEPTH, RATE],
            Kind::Flac => &[FLAC_DEPTH, RATE],
            Kind::Mp3 => &[MP3_QUALITY, MP3_RATE],
            Kind::Opus => &[OPUS_BITRATE],
        }
    }

    /// What the dialog says under the choices.
    pub fn note(self) -> &'static str {
        match self {
            Kind::Wav => "Uncompressed: every program and player opens it. At most 4 GB a file.",
            Kind::Flac => "Lossless: the same sound as WAV in about half the space.",
            Kind::Mp3 => "Lossy and small: plays everywhere.",
            Kind::Opus => {
                "Lossy and smaller than MP3 at the same quality; always at 48 kHz. Current browsers and most players open it."
            }
        }
    }
}

/// One choice: a list of options, kept in the settings by their keys.
#[derive(Debug, PartialEq, Eq)]
pub struct Field {
    /// Its key in the format's settings object, and its widget's id.
    pub key: &'static str,
    /// Its label in the dialog.
    pub label: &'static str,
    /// Each option's key and title.
    pub options: &'static [(&'static str, &'static str)],
    /// The option picked until the user picks one.
    pub default: usize,
}

const WAV_DEPTH: Field = Field {
    key: "depth",
    label: "Bit depth",
    options: &[
        ("16", "16-bit (CD; smallest files)"),
        ("24", "24-bit (recommended)"),
        ("32f", "32-bit float (for further mixing)"),
    ],
    default: 1,
};

const FLAC_DEPTH: Field = Field {
    key: "depth",
    label: "Bit depth",
    options: &[
        ("16", "16-bit (CD; smallest files)"),
        ("24", "24-bit (recommended)"),
    ],
    default: 1,
};

const RATE: Field = Field {
    key: "rate",
    label: "Rate",
    options: &[
        ("44100", "44.1 kHz (CD)"),
        ("48000", "48 kHz (recommended)"),
    ],
    default: 1,
};

const MP3_RATE: Field = Field {
    key: "rate",
    label: "Rate",
    options: &[("44100", "44.1 kHz (recommended)"), ("48000", "48 kHz")],
    default: 0,
};

/// MP3's quality: variable bitrates by LAME's V numbers, or a constant
/// bitrate in kbps.
const MP3_QUALITY: Field = Field {
    key: "quality",
    label: "Quality",
    options: &[
        ("v0", "Variable, highest (about 245 kbps)"),
        ("v2", "Variable, high (about 190 kbps; recommended)"),
        ("v4", "Variable, medium (about 165 kbps)"),
        ("320", "Constant, 320 kbps"),
        ("256", "Constant, 256 kbps"),
        ("192", "Constant, 192 kbps"),
        ("128", "Constant, 128 kbps"),
    ],
    default: 1,
};

/// Opus's bitrate in kbps.
const OPUS_BITRATE: Field = Field {
    key: "bitrate",
    label: "Bitrate",
    options: &[
        ("96", "96 kbps (small)"),
        ("128", "128 kbps"),
        ("160", "160 kbps (recommended)"),
        ("192", "192 kbps"),
        ("256", "256 kbps (highest)"),
    ],
    default: 2,
};

/// The format's object in the plugin's settings, or an empty one.
pub fn object(settings: &Map<String, Value>, kind: Kind) -> Map<String, Value> {
    settings
        .get(kind.id())
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

/// The option `field` has in `object`: the one saved, or its default.
pub fn picked(object: &Map<String, Value>, field: &Field) -> usize {
    object
        .get(field.key)
        .and_then(Value::as_str)
        .and_then(|k| field.options.iter().position(|(key, _)| *key == k))
        .unwrap_or(field.default)
}

/// The key of the option `field` has in `object`.
pub fn key(object: &Map<String, Value>, field: &Field) -> &'static str {
    field.options[picked(object, field)].0
}

/// The picks of `kind` in the plugin's settings, each choice's key by
/// the choice's key.
pub fn picks(settings: &Map<String, Value>, kind: Kind) -> Picks {
    let object = object(settings, kind);
    Picks(
        kind.fields()
            .iter()
            .map(|f| (f.key, key(&object, f)))
            .collect(),
    )
}

/// One format's picks.
#[derive(Debug, PartialEq, Eq)]
pub struct Picks(Vec<(&'static str, &'static str)>);

impl Picks {
    /// The option picked for the choice `key`.
    pub fn get(&self, key: &str) -> &'static str {
        self.0
            .iter()
            .find(|(k, _)| *k == key)
            .map_or("", |(_, v)| *v)
    }

    /// The rate picked, in frames a second: 48 kHz when the format has no
    /// choice of rate.
    pub fn rate(&self) -> u32 {
        self.get("rate").parse().unwrap_or(48_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxiguitar_plugin::serde_json::json;

    #[test]
    fn every_choice_has_options_and_a_default_among_them() {
        for kind in Kind::ALL {
            assert_eq!(Kind::of(kind.id()), Some(kind));
            assert_eq!(Kind::of(kind.options()), Some(kind));
            for f in kind.fields() {
                assert!(f.default < f.options.len(), "{kind:?} {}", f.key);
            }
        }
        assert_eq!(Kind::of("ogg"), None);
    }

    #[test]
    fn picks_come_from_the_settings_or_the_defaults() {
        let none = Map::new();
        let wav = picks(&none, Kind::Wav);
        assert_eq!((wav.get("depth"), wav.rate()), ("24", 48_000));
        assert_eq!(picks(&none, Kind::Mp3).rate(), 44_100);
        assert_eq!(picks(&none, Kind::Opus).rate(), 48_000, "always 48 kHz");
        let settings = json!({
            "wav": {"depth": "16", "rate": "44100"},
            "flac": {"depth": "32f", "rate": 96000},
        });
        let settings = settings.as_object().unwrap();
        let wav = picks(settings, Kind::Wav);
        assert_eq!((wav.get("depth"), wav.rate()), ("16", 44_100));
        // What a format doesn't offer, or a value of the wrong kind, is
        // the default.
        let flac = picks(settings, Kind::Flac);
        assert_eq!((flac.get("depth"), flac.rate()), ("24", 48_000));
    }
}
