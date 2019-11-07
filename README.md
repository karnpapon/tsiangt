# tsiangt

### terminal music player
เสียง ( \siang\ ) means "sound" in Thai language.

### Usage
- set directory by `-d <directory>`, otherwise tsiangt will automatically use default music's directory path, depends on OS. see [this](https://docs.rs/dirs/2.0.2/dirs/fn.audio_dir.html) for more details.

#### Keybinding

you'll feel at home if you're familiar with vim keybinding.

Description |  Operation | note
--- | --- | ---
add song to playlist (at library page) | `enter` or `return` |
play (at playlist page) | `enter` or `return` |
stop | `s` |
switch to playlist tab | `1` |
switch to library tab | `2` |
switch to search tab | `3` | soon..
resume / pause | `spacebar` |
move-left | `h` or `arrow-left` |
move-right | `l` or `arrow-right` |
move-up | `k` or `arrow-up` |
move-down | `j` or `arrow-down` |

built with
- [tui-rs](https://github.com/fdehau/tui-rs)
- [termion](https://github.com/redox-os/termion)
- [rodio](https://github.com/RustAudio/rodio)

<img src="img/ss.png?sanitize=true">
