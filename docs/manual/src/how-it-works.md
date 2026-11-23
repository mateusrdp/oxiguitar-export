# Architecture and strategy

## How it fits into OxiGuitar

The plugin is an *exporter*. OxiGuitar does all the playing: it renders the score into a stream of sound through the instrument, effect and output plugins, exactly as it would play it, only as fast as the processor allows. The plugin receives that stream and does two things: it encodes it in the chosen format and writes the file.

- **WAV** is our own code. 16-bit samples get triangular dither, one step wide, from a fixed generator, so the same song always exports to the same file.
- **FLAC** goes through [rusty_flac](https://github.com/Remade-With-Rust/rusty_flac), **MP3** through [rusty_mp3](https://github.com/Remade-With-Rust/remade_ffmpeg_rs), and **Opus** through [opus-pure](https://github.com/stephenberry/opus-pure). All three are pure Rust, so the plugin has no C libraries to install, and all are used as published.
- MP3 files carry a LAME Info/Xing tag, and Opus files the pre-skip in their header, so players can cut the encoder's padding and play gaplessly.
- A file is written under a temporary name and takes its real name only when it is whole. A cancelled or failed export leaves nothing behind.

Each format's own choices (bit depth, rate, bitrate) are shown in the export dialog only while that format is picked. **Export…** keeps them for next time; **Cancel** forgets them. `oxiguitar-cli export SONG.gp5 SONG.flac` uses the last kept choices.

## Current limitations

- **Two sample rates.** 44.1 and 48 kHz, nothing higher. Opus is always 48 kHz, by design of the format.
- **No tags.** The title, artist and so on aren't written into the file yet.
- **No stems.** One file for the whole mix; exporting each track to its own file means muting the others by hand.
- **No normalising.** The file is as loud as the mix. Turn the tracks up or down in the mixer first.
- **WAV files stop at 4 GB**, over three hours of sound. That is the format's own limit.

## Left out on purpose

- **C libraries such as LAME, libFLAC or libopus.** They are excellent, but bundling them on every system, or asking you to install them, is exactly the kind of setup OxiGuitar avoids. Pure Rust encoders build everywhere the plugin does.
- **More formats** (AAC, Ogg Vorbis, AIFF). WAV, FLAC, MP3 and Opus cover sharing, editing and archiving. Another format is one more encoder to maintain, so it waits until someone needs it.
- **Uploading anywhere.** OxiGuitar never connects to the internet. The file is yours to send.

## Where you can help

- **Tags** for MP3, FLAC and Opus: title, artist, album from the score's own information.
- **Stems**: one file per track, in one go.
- **Listening tests** of the encoders' quality settings against other encoders.

Every change needs its tests and a row on the [Features](features.md) page; the [Testing](testing.md) page says how to run them.
