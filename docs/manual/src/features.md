# Features and their tests

Everything the audio export plugin does, and the tests that check it. The test names are the ones `cargo test` prints. `scripts/test-all.py` fails when a feature here names no test, or names a test that doesn't exist, so a new feature gets its row and its tests in the same pull request (the [Testing](testing.md) page says how to run them).

OxiGuitar itself renders the song (the selected bars, the practice speed, the metronome, every track with its sound, amp and mixer), faster than real time and without a sound card, and writes the file under a temporary name until it is whole. That part is tested in OxiGuitar's own repository. This plugin turns the sound into each format's bytes.

| Feature | What it does | Tests |
| --- | --- | --- |
| Four formats | WAV, FLAC, MP3 and Opus, each an exporter of its own with its options. | `tests::every_format_has_its_exporter_and_options`, `tests::each_format_starts_at_its_rate_and_finishes` |
| WAV | 16-bit with dither, 24-bit or 32-bit float, at 44.1 or 48 kHz; the header names the format and the length; past four gigabytes the export stops. | `wav::tests::samples_become_bytes_at_each_depth`, `wav::tests::headers_name_the_format_and_the_length`, `wav::tests::past_four_gigabytes_the_export_stops`, `pcm::tests::full_scale_and_dither` |
| FLAC | Lossless, 16-bit with dither or 24-bit, streamed in pieces. | `flac::tests::a_stream_in_pieces`, `pcm::tests::full_scale_and_dither` |
| MP3 | Variable bitrate V0, V2 or V4, or constant 128 to 320 kbps; gapless; streamed in pieces. | `mp3::tests::a_stream_in_pieces`, `mp3::tests::qualities` |
| Opus | 96 to 256 kbps, always 48 kHz, streamed in pieces. | `opus::tests::a_stream_in_pieces` |
| Each format's choices | Every choice has options and a default among them; picks come from the settings or the defaults; the dialog's changes are a draft, kept with Export and forgotten with Cancel. Drawn by `ui/export.slint`, one component for the four formats, which compiles with what the plugin sets and hears. | `choices::tests::every_choice_has_options_and_a_default_among_them`, `choices::tests::picks_come_from_the_settings_or_the_defaults`, `tests::the_options_change_a_draft_until_they_close`, `tests::the_slint_file_has_what_the_options_set_and_hear` |
| A working plugin file | The `.oxp` file keeps the plugin contract, with this repository's SDK version and with the newest release of OxiGuitar. | `contract::the_plugin_file_keeps_the_contract`, `oxiguitar_plugin_checks::the_manifest_matches_the_plugin`, `stage:contract (latest core)` |
