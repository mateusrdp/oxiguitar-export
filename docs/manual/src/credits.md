# Credits

Thank you to the people whose work this plugin is built on.

## Code we use

| Project | Authors | What it does here | Licence |
| --- | --- | --- | --- |
| [OxiGuitar's plugin SDK](https://github.com/mateusrdp/oxiguitar) (`oxiguitar-plugin`, and `oxiguitar-host` and `oxiguitar-plugin-test` for the tests) | Mateus Rocha de Paula and the OxiGuitar contributors | The plugin interface, and OxiGuitar's own host to test against | MIT OR Apache-2.0 |
| [rusty_flac](https://github.com/Remade-With-Rust/rusty_flac) | Mata Network | FLAC encoding | Apache-2.0 |
| [rusty_mp3](https://github.com/Remade-With-Rust/remade_ffmpeg_rs) | Mata Network | MP3 encoding | Apache-2.0 |
| [opus-pure](https://github.com/stephenberry/opus-pure) | Stephen Berry and restsend.com; a port of libopus (Xiph.Org, Skype Limited, Octasic and others) | Opus encoding | BSD-3-Clause |
| [Rust](https://www.rust-lang.org/) and its standard library | The Rust Project | Everything else | MIT OR Apache-2.0 |

The WAV writer and the dither are our own code.

## Standards

The formats themselves are the work of many people: [FLAC](https://xiph.org/flac/) and the Ogg container from Xiph.Org, [Opus](https://opus-codec.org/) from the IETF and Xiph.Org, MP3 from the Fraunhofer Institute and MPEG, and the gapless Info tag from the [LAME](https://lame.sourceforge.io/) project.

## Tools

| Tool | Use |
| --- | --- |
| [mdBook](https://github.com/rust-lang/mdBook) | Builds this manual |
| [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) | Checks the licences of everything we build with |

## Thanks

To the authors of the pure Rust encoders, who made it possible to export to every format with no C library at all, and to everyone who exports a song and tells us how it came out.
