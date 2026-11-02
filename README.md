# Tiles

The classic **2048** puzzle as a native Linux desktop app, written in Rust with
[GTK4](https://gtk-rs.org) and [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/).

Styled after [play2048.co](https://play2048.co), with animated tile slides and
merge pops.

## Features

- Classic 4×4 2048 gameplay with smooth move animations (slide + merge pop)
- Three color modes: **Light**, **Sepia** (the classic play2048.co cream) and
  **Dark** — switchable from the app menu and remembered between runs
- Keyboard (arrows, WASD, HJKL), on-screen controls and mouse/touch swipes
- Score + persisted best score (`$XDG_DATA_HOME/tiles/best`)
- Win / game-over overlays; keep playing after reaching 2048
- GNOME-style app menu (hamburger) with an About window: logo, website,
  issue tracker, credits and the MIT license

## Build from source

Dependencies: a Rust toolchain (1.85+), `glib-compile-resources` (glib2 /
libglib2.0-dev) plus the GTK 4 and libadwaita development packages.

```sh
# Arch Linux
sudo pacman -S --needed gtk4 libadwaita
# Debian / Ubuntu
sudo apt install libgtk-4-dev libadwaita-1-dev

cargo build --release
./target/release/tiles
```

## Tests

The game logic is covered by unit tests that run without a display:

```sh
cargo test
```

## Packages

The app id is `io.github.hakergeniusz.Tiles`. Everything lives under
`packaging/`:

| Target  | File(s)                                   | Build                                                                 |
| ------- | ----------------------------------------- | --------------------------------------------------------------------- |
| Debian  | `packaging/deb/build-deb.sh`              | `cargo build --release` then `packaging/deb/build-deb.sh` (needs `ar`) |
| Fedora  | `packaging/fedora/tiles.spec`             | `rpmbuild -bb packaging/fedora/tiles.spec`                            |
| Arch    | `packaging/arch/PKGBUILD`                 | `makepkg -p packaging/arch/PKGBUILD` (builds from the `v0.4.0` tag)   |
| Flatpak | `packaging/flatpak/io.github.hakergeniusz.Tiles.yml` | `flatpak-builder --user --install --force-clean build-dir packaging/flatpak/io.github.hakergeniusz.Tiles.yml` (needs `org.gnome.Platform//48` + Sdk) |

Every package installs the binary as `/usr/bin/tiles`, the desktop entry, the
hicolor icon and the AppStream metainfo.

## Controls

| Input                         | Action                       |
| ----------------------------- | ---------------------------- |
| Arrow keys / WASD / HJKL      | Slide tiles                  |
| Mouse drag / touch swipe      | Slide tiles                  |
| On-screen arrows              | Slide tiles (mouse/touch)    |
| `New Game` button             | Restart the round            |
| Hamburger menu → Color Mode   | Light / Sepia / Dark mode    |
| Hamburger menu → About        | About window                 |

## License

Distributed under the [MIT license](./LICENSE).
