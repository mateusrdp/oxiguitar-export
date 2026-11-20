# Audio export plugin

This plugin adds audio formats to **File → Export Audio…**. OxiGuitar plays the whole song, or the selected bars, through its instruments, amps and mixer, faster than real time. This plugin writes that sound as a WAV, FLAC, MP3 or Opus file.

Until version 0.1.2 it was the WAV plugin. It replaces that plugin, so OxiGuitar uses this one even if the old file is still installed. You can remove the old package, `oxiguitar-wav`.

This work depends on the MIDI plugin, which makes the sound that gets written. See [its manual](https://mateusrdp.github.io/oxiguitar-midi/).
