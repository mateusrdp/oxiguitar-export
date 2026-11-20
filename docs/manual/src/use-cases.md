# What you can do with it

- Make an audio file of a song to share, practise with, or load into another program.
- Export only the selected bars, for a part you want to loop.
- Export at your practice speed. The pitch stays the same.
- Include the metronome's clicks, if you want them.
- Export without a sound card. A three-minute song takes seconds.

The export dialog shows each format's own choices under its **Format** list. They are kept with **Export…** and forgotten with **Cancel**.

| Format | Choices | Default |
| --- | --- | --- |
| WAV | 16-bit (with dither), 24-bit or 32-bit float; 44.1 or 48 kHz | 24-bit, 48 kHz |
| FLAC, lossless | 16-bit (with dither) or 24-bit; 44.1 or 48 kHz | 24-bit, 48 kHz |
| MP3 | variable bitrate V0, V2 or V4, or constant 128 to 320 kbps; 44.1 or 48 kHz; gapless | V2, about 190 kbps, 44.1 kHz |
| Opus (`.opus`) | 96 to 256 kbps; always 48 kHz | 160 kbps |

From the command line, `oxiguitar-cli export SONG.gp5 SONG.flac` does the same, with the choices last kept.

A file is written under a temporary name and takes its real name only when it is whole, so a cancelled or failed export leaves nothing behind.
