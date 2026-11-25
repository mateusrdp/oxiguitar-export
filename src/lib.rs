// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Mateus Rocha de Paula and the OxiGuitar contributors

//! OxiGuitar's audio export plugin (plugin interface 1.16; M4 slice 7,
//! PLAY-7; mateusrdp/oxiguitar#313): the formats of File → Export Audio….
//! OxiGuitar renders the score and hands the sound over in pieces; this
//! plugin encodes it, and OxiGuitar writes the bytes to the file.
//!
//! - Four formats, each an exporter: WAV ([`wav`]), FLAC ([`flac`]), MP3
//!   ([`mp3`]) and Opus in Ogg ([`opus`]), every encoder pure Rust.
//! - Each format's choices are a view of its own ([`Options`],
//!   `ui/export.slint`), which the export dialog shows under its list of
//!   formats while the format is picked (the exporter's `options`). It
//!   changes a draft: OK on the export dialog keeps the picks in the
//!   plugin's settings before the export starts, Cancel forgets them
//!   ([`choices`]).
//! - It was the WAV plugin (`oxiguitar.wav`) until 0.1.2: its manifest
//!   `replaces` that id, so a leftover `wav.oxp` counts as an older
//!   version of this one.

use std::sync::Mutex;

use oxiguitar_plugin::serde_json::{Map, Value};
use oxiguitar_plugin::{
    AudioExporter, Error, ErrorKind, ExportEnd, ExportStart, Host, Plugin, TrackView, Ui,
    Value as View,
};

pub mod choices;
pub mod flac;
pub mod mp3;
pub mod opus;
pub mod pcm;
pub mod wav;

use choices::Kind;

/// An export under way, in one of the formats.
#[derive(Debug)]
enum Encoding {
    Wav(wav::Wav),
    Flac(Box<flac::Flac>),
    Mp3(Box<mp3::Mp3>),
    Opus(Box<opus::Opus>),
}

impl Encoding {
    /// Starts an export of `kind` with the picks the settings hold: the
    /// rate to render at and the file's first bytes.
    fn start(kind: Kind, settings: &Map<String, Value>) -> Result<(Encoding, u32, Vec<u8>), Error> {
        let picks = choices::picks(settings, kind);
        let rate = picks.rate();
        Ok(match kind {
            Kind::Wav => {
                let (w, header) = wav::Wav::start(wav::Depth::of(picks.get("depth")), rate);
                (Encoding::Wav(w), rate, header)
            }
            Kind::Flac => {
                let bits = if picks.get("depth") == "16" { 16 } else { 24 };
                let (f, header) = flac::Flac::start(bits, rate)?;
                (Encoding::Flac(Box::new(f)), rate, header)
            }
            Kind::Mp3 => (
                Encoding::Mp3(Box::new(mp3::Mp3::start(picks.get("quality"), rate))),
                rate,
                Vec::new(),
            ),
            Kind::Opus => {
                let kbps = picks.get("bitrate").parse().unwrap_or(160);
                let (o, header) = opus::Opus::start(kbps)?;
                (Encoding::Opus(Box::new(o)), opus::RATE, header)
            }
        })
    }

    fn samples(&mut self, interleaved: &[f32]) -> Result<Vec<u8>, Error> {
        match self {
            Encoding::Wav(w) => w.samples(interleaved),
            Encoding::Flac(f) => f.samples(interleaved),
            Encoding::Mp3(m) => m.samples(interleaved),
            Encoding::Opus(o) => o.samples(interleaved),
        }
    }

    /// The file's last bytes, and what to write over its start.
    fn finish(self) -> Result<ExportEnd, Error> {
        let (data, start) = match self {
            Encoding::Wav(w) => (Vec::new(), Some(w.finish())),
            Encoding::Flac(f) => {
                let (tail, header) = f.finish();
                (tail, Some(header))
            }
            Encoding::Mp3(m) => m.finish()?,
            Encoding::Opus(o) => (o.finish()?, None),
        };
        Ok(ExportEnd { data, start })
    }
}

/// One format's exporter.
#[derive(Debug)]
pub struct Exporter {
    kind: Kind,
    host: Host,
    export: Mutex<Option<Encoding>>,
}

impl Exporter {
    fn export(&self) -> std::sync::MutexGuard<'_, Option<Encoding>> {
        self.export.lock().unwrap_or_else(|e| e.into_inner())
    }
}

fn not_started() -> Error {
    Error::new(ErrorKind::Failed, "no export was started")
}

impl AudioExporter for Exporter {
    fn start(&self) -> Result<ExportStart, Error> {
        let settings = self.host.settings().unwrap_or_default();
        let (encoding, rate, data) = Encoding::start(self.kind, &settings)?;
        *self.export() = Some(encoding);
        Ok(ExportStart { rate, data })
    }

    fn samples(&self, interleaved: &[f32]) -> Result<Vec<u8>, Error> {
        self.export()
            .as_mut()
            .ok_or_else(not_started)?
            .samples(interleaved)
    }

    fn finish(&self, cancelled: bool) -> Result<ExportEnd, Error> {
        let encoding = self.export().take().ok_or_else(not_started)?;
        if cancelled {
            return Ok(ExportEnd::default());
        }
        encoding.finish()
    }
}

/// A format's options: the view the export dialog shows under its list of
/// formats. It changes a draft of the plugin's settings, which OK writes
/// and Cancel forgets.
#[derive(Debug)]
pub struct Options {
    kind: Kind,
    host: Host,
    /// The settings as the dialog has them, once something was picked.
    draft: Mutex<Option<Map<String, Value>>>,
}

impl Options {
    /// The settings as the dialog has them: its draft, or what is saved.
    fn current(&self) -> Map<String, Value> {
        let draft = self.draft.lock().ok().and_then(|d| d.as_ref().cloned());
        draft.unwrap_or_else(|| self.host.settings().unwrap_or_default())
    }

    /// The view's properties: a choice per option, then the note.
    pub fn properties(&self) -> Vec<(String, View)> {
        let object = choices::object(&self.current(), self.kind);
        let fields = self
            .kind
            .fields()
            .iter()
            .map(|f| {
                View::Struct(vec![
                    ("label".into(), View::String(f.label.into())),
                    (
                        "options".into(),
                        View::List(
                            f.options
                                .iter()
                                .map(|(_, t)| View::String((*t).into()))
                                .collect(),
                        ),
                    ),
                    (
                        "picked".into(),
                        View::Number(choices::picked(&object, f) as f64),
                    ),
                ])
            })
            .collect();
        vec![
            ("fields".into(), View::List(fields)),
            ("note".into(), View::String(self.kind.note().into())),
        ]
    }

    /// The user picked the option `option` of the field `field`.
    pub fn choose(&self, field: usize, option: usize) -> Result<(), Error> {
        let field = self
            .kind
            .fields()
            .get(field)
            .ok_or_else(|| Error::not_found(format!("no field {field}")))?;
        let (key, _) = field
            .options
            .get(option)
            .ok_or_else(|| Error::new(ErrorKind::Failed, format!("no choice {option}")))?;
        let mut settings = self.current();
        let mut object = choices::object(&settings, self.kind);
        object.insert(field.key.into(), (*key).into());
        settings.insert(self.kind.id().into(), object.into());
        if let Ok(mut draft) = self.draft.lock() {
            *draft = Some(settings);
        }
        Ok(())
    }

    /// OK writes the picks, if any were made; Cancel forgets them.
    pub fn closed(&self, ok: bool) -> Result<(), Error> {
        let draft = self.draft.lock().ok().and_then(|mut d| d.take());
        match draft {
            Some(settings) if ok => self.host.set_settings(&settings),
            _ => Ok(()),
        }
    }
}

/// A callback's argument `i`, a whole number from 0.
fn index(args: &[View], i: usize) -> Option<usize> {
    match args.get(i) {
        Some(View::Number(n)) if *n >= 0.0 && n.fract() == 0.0 => Some(*n as usize),
        _ => None,
    }
}

/// The plugin.
#[derive(Debug)]
pub struct ExportPlugin {
    exporters: Vec<Exporter>,
    options: Vec<Options>,
}

impl Plugin for ExportPlugin {
    fn start(host: Host) -> Result<ExportPlugin, Error> {
        Ok(ExportPlugin {
            exporters: Kind::ALL
                .into_iter()
                .map(|kind| Exporter {
                    kind,
                    host: host.clone(),
                    export: Mutex::default(),
                })
                .collect(),
            options: Kind::ALL
                .into_iter()
                .map(|kind| Options {
                    kind,
                    host: host.clone(),
                    draft: Mutex::new(None),
                })
                .collect(),
        })
    }

    fn audio_exporter(&self, id: &str) -> Option<&dyn AudioExporter> {
        let kind = Kind::ALL.into_iter().find(|k| k.id() == id)?;
        self.exporters
            .iter()
            .find(|e| e.kind == kind)
            .map(|e| e as &dyn AudioExporter)
    }

    fn ui(&self) -> Option<&dyn Ui> {
        Some(self)
    }
}

impl ExportPlugin {
    /// The options of the format whose options' view is `view`.
    fn options_of(&self, view: &str) -> Option<&Options> {
        let kind = Kind::ALL.into_iter().find(|k| k.options() == view)?;
        self.options.iter().find(|o| o.kind == kind)
    }
}

/// The four formats' options, as views of `ui/export.slint`.
impl Ui for ExportPlugin {
    fn shown(&self, view: &str, _track: Option<&TrackView>) -> Result<(), Error> {
        match self.options_of(view) {
            Some(o) => o.host.set_view(view, o.properties()),
            None => Ok(()),
        }
    }

    fn callback(&self, view: &str, callback: &str, args: Vec<View>) -> Result<(), Error> {
        let Some(o) = self.options_of(view) else {
            return Ok(());
        };
        if callback != "chosen" {
            return Ok(());
        }
        let (Some(field), Some(option)) = (index(&args, 0), index(&args, 1)) else {
            return Ok(());
        };
        o.choose(field, option)?;
        o.host.set_view(view, o.properties())
    }

    fn closed(&self, view: &str, accepted: bool) -> Result<(), Error> {
        match self.options_of(view) {
            Some(o) => o.closed(accepted),
            None => Ok(()),
        }
    }
}

oxiguitar_plugin::export_plugin!(ExportPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    fn options(kind: Kind) -> Options {
        Options {
            kind,
            host: Host::detached(),
            draft: Mutex::new(None),
        }
    }

    fn selected(o: &Options, key: &str) -> u32 {
        let field = o.kind.fields().iter().position(|f| f.key == key).unwrap();
        let p = o.properties();
        let View::List(fields) = &p[0].1 else {
            panic!()
        };
        let View::Struct(f) = &fields[field] else {
            panic!()
        };
        match f.iter().find(|(n, _)| n == "picked") {
            Some((_, View::Number(n))) => *n as u32,
            other => panic!("{other:?}"),
        }
    }

    fn field(kind: Kind, key: &str) -> usize {
        kind.fields().iter().position(|f| f.key == key).unwrap()
    }

    /// The options show their picks at once and keep them in a draft
    /// until they close: Cancel forgets them (a host that keeps no
    /// settings, so what OK writes is checked through OxiGuitar's host in
    /// tests/contract.rs).
    #[test]
    fn the_options_change_a_draft_until_they_close() {
        let wav = options(Kind::Wav);
        assert_eq!(selected(&wav, "depth"), 1, "24-bit by default");
        wav.choose(field(Kind::Wav, "depth"), 0).unwrap();
        wav.choose(field(Kind::Wav, "rate"), 0).unwrap();
        assert_eq!((selected(&wav, "depth"), selected(&wav, "rate")), (0, 0));
        let draft = wav.current();
        assert_eq!(
            draft["wav"],
            oxiguitar_plugin::serde_json::json!({"depth": "16", "rate": "44100"})
        );
        wav.closed(false).unwrap();
        assert_eq!((selected(&wav, "depth"), selected(&wav, "rate")), (1, 1));
        assert!(wav.choose(9, 0).is_err());
        assert!(wav.choose(field(Kind::Wav, "depth"), 9).is_err());
        assert_eq!(selected(&options(Kind::Mp3), "quality"), 1, "V2");
        assert_eq!(selected(&options(Kind::Opus), "bitrate"), 2, "160 kbps");
    }

    #[test]
    fn every_format_has_its_exporter_and_options() {
        let plugin = ExportPlugin::start(Host::detached()).unwrap();
        for kind in Kind::ALL {
            assert!(plugin.audio_exporter(kind.id()).is_some());
            assert!(plugin.options_of(kind.options()).is_some());
        }
        assert!(plugin.audio_exporter("ogg").is_none());
        assert!(plugin.options_of("wav").is_none());
    }

    /// The file compiles as OxiGuitar compiles it, with every property and
    /// callback the options use.
    #[test]
    fn the_slint_file_has_what_the_options_set_and_hear() {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/export.slint");
        let compiled = oxiguitar_plugin_test::compile_ui_file(&file).unwrap();
        let c = compiled
            .components
            .iter()
            .find(|c| c.name == "FormatOptions")
            .unwrap();
        for (name, _) in options(Kind::Flac).properties() {
            assert!(c.properties.contains(&name), "{name}");
        }
        assert!(c.callbacks.iter().any(|p| p == "chosen"));
    }

    #[test]
    fn each_format_starts_at_its_rate_and_finishes() {
        let none = Map::new();
        for (kind, rate) in [
            (Kind::Wav, 48_000),
            (Kind::Flac, 48_000),
            (Kind::Mp3, 44_100),
            (Kind::Opus, 48_000),
        ] {
            let (mut e, r, _) = Encoding::start(kind, &none).unwrap();
            assert_eq!(r, rate, "{kind:?}");
            e.samples(&[0.1; 4096]).unwrap();
            e.finish().unwrap();
        }
    }
}
