# Audio export plugin

This plugin turns a song into an audio file you can share, practise with or load into another program. **File → Export Audio…** plays the whole song, or the selected bars, through every instrument, amp and mixer setting you have, a lot faster than real time, and this plugin writes the result as a **WAV**, **FLAC**, **MP3** or **Opus** file. [How to use it](use-cases.md) shows the dialog.

It replaces the older WAV plugin (up to version 0.1.2): OxiGuitar uses this one even if the old file is still installed, and you can remove the old package, `oxiguitar-wav`.

This plugin needs the MIDI plugin, which makes the sound that gets written. See [its manual](https://mateusrdp.github.io/oxiguitar-midi/).

## A little about digital audio

Sound in a computer is a long list of numbers, *samples*, each one the position of the speaker cone at one instant. Two settings decide how fine that list is:

- **Sample rate**: how many samples a second. CDs use 44,100 (44.1 kHz); video and most audio software use 48,000 (48 kHz). Both capture everything people can hear, so pick the one the place you are sending the file to expects.
- **Bit depth**: how precisely each sample is stored. 16 bits is CD quality. 24 bits leaves more room for later editing. 32-bit float is what audio programs use inside and can't clip.

Going down to 16 bits rounds every sample a little, which on quiet passages can leave a faint, gritty distortion. **Dither** adds a whisper of random noise before the rounding so the error becomes a smooth, inaudible hiss instead. The plugin does this by itself whenever it writes 16 bits.

## Lossless and lossy

- **WAV** stores the samples as they are. Every program reads it, and files are big: about 10 MB a minute at 16 bits and 44.1 kHz in stereo.
- **FLAC** is *lossless*: it packs the same samples into about half the space, like a ZIP file for sound, and gives back exactly the same numbers when played.
- **MP3** and **Opus** are *lossy*: they throw away what the ear is least likely to notice, and files come out around a tenth the size of WAV or smaller. MP3 plays everywhere. Opus, the newer one, sounds better at the same size; it is also the codec web browsers use for voice and video calls. How much they keep is the **bitrate**, in kilobits per second (kbps); *variable bitrate* (VBR, MP3's V0 to V4) spends more on busy passages and less on quiet ones.

A lossy file is fine for listening and sharing. To edit the sound later, or to keep a master copy, use WAV or FLAC.

## Gapless

MP3 and Opus encoders normally add a little silence at the start and end of a file. The plugin records how much in the file, so a player that understands it (most do) starts and ends exactly where the song does, and a loop plays without a gap.
