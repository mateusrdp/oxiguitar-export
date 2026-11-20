# Architecture and strategy

The plugin is an exporter. OxiGuitar does the playing: it renders the score into a stream. The plugin only encodes the stream and writes the file.

The strategy is one plugin for all the audio formats, with each format's choices shown only while that format is picked. MP3 and Opus files start and end exactly where the song does, so a song loops without a gap in a player that can.
