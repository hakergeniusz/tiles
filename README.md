# GTK2048

The classic **2048** puzzle as a native Linux desktop app, written in Rust with
[GTK4](https://gtk-rs.org) and [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/).

Styled after [play2048.co](https://play2048.co), with animated tile slides and
merge pops.

## Features

- Classic 4×4 2048 gameplay with smooth move animations (slide + merge pop)
- Three color modes: **Light**, **Sepia** (the classic play2048.co cream) and
  **Dark** — switchable from the header bar and remembered between runs
- Keyboard (arrows, WASD, HJKL) and on-screen controls
- Score + persisted best score (`$XDG_DATA_HOME/gtk2048/best`)
- Win / game-over overlays; keep playing after reaching 2048
- About dialog under **About** in the header bar

## Build

Dependencies: a Rust toolchain (1.85+) plus the GTK 4 and libadwaita
development packages.

```sh
# Arch Linux
sudo pacman -S --needed gtk4 libadwaita
# Debian / Ubuntu
sudo apt install libgtk-4-dev libadwaita-1-dev
```

```sh
cargo build --release
./target/release/gtk2048
```

## Tests

The game logic is covered by unit tests that run without a display:

```sh
cargo test
```

## Controls

| Input                    | Action                    |
| ------------------------ | ------------------------- |
| Arrow keys / WASD / HJKL | Slide tiles               |
| On-screen arrows         | Slide tiles (mouse/touch) |
| `New Game` button        | Restart the round         |
| Header-bar drop-down     | Light / Sepia / Dark mode |

## License

Distributed under the [MIT license](./LICENSE).
