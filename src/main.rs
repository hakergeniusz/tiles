//! Tiles — the classic 2048 puzzle, built with GTK4 and libadwaita.

mod board_widget;
mod game;
mod theme;

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use gtk::gdk;
use gtk::gio;
use gtk::glib;
use gtk::{Align, Button, Label, Orientation, Overlay};

use board_widget::BoardWidget;
use game::{Direction, Game, SIZE};
use theme::PALETTES;

const APP_ID: &str = "io.github.hakergeniusz.Tiles";

/// How often (ms) to re-check for the win / game-over overlay while the
/// move animation is still running.
const OVERLAY_POLL_MS: u64 = 100;

/// The mode that is active without a saved preference: play2048.co cream.
const DEFAULT_THEME: usize = 1;

fn main() -> glib::ExitCode {
    register_resources();
    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run()
}

/// Register the bundled icons (app logo for About and the taskbar).
fn register_resources() {
    let bytes: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tiles.gresource"));
    let resource =
        gio::Resource::from_data(&glib::Bytes::from(bytes)).expect("bundled GResource is valid");
    gio::resources_register(&resource);
}

/// The widgets the game state needs to reach.
struct App {
    game: RefCell<Game>,
    best: Cell<u32>,
    board: BoardWidget,
    score_label: Label,
    best_label: Label,
    overlay_box: gtk::Box,
    overlay_title: Label,
    keep_btn: Button,
    css_provider: gtk::CssProvider,
}

impl App {
    /// Update the score labels and persist a new best score.
    fn update_scores(&self) {
        let game = self.game.borrow();
        if game.score > self.best.get() {
            self.best.set(game.score);
            save_best(game.score);
        }
        self.score_label.set_text(&game.score.to_string());
        self.best_label.set_text(&self.best.get().to_string());
    }

    /// Play one move, animate it, and react to a win or the end of the game.
    fn do_move(self: Rc<Self>, dir: Direction) {
        if self.overlay_box.is_visible() {
            return;
        }
        {
            let mut game = self.game.borrow_mut();
            if game.over {
                return;
            }
            // If the previous animation is still running, snap it to its end
            // so this move starts from the settled board.
            self.board.finish_animation();
            let Some(effect) = game.move_dir_with_effects(dir) else {
                return;
            };
            let spawn = game.spawn_tile();
            game.over = game.is_game_over();
            let final_board = game.board;
            drop(game);

            self.board.play_move(&effect, spawn, final_board);
        }
        self.update_scores();
        Self::schedule_overlay_check(&self);
    }

    /// Show the win / game-over overlay once the move animation is done.
    fn schedule_overlay_check(state: &Rc<Self>) {
        let state = Rc::clone(state);
        glib::timeout_add_local(Duration::from_millis(OVERLAY_POLL_MS), move || {
            if state.board.is_animating() {
                return glib::ControlFlow::Continue;
            }
            let (won, keep_playing, over) = {
                let game = state.game.borrow();
                (game.won, game.keep_playing, game.over)
            };
            if won && !keep_playing {
                state.show_overlay("You win! 🎉", true);
            } else if over {
                state.show_overlay("Game over!", false);
            }
            glib::ControlFlow::Break
        });
    }

    fn new_game(&self) {
        let (board, spawns) = {
            *self.game.borrow_mut() = Game::new();
            let game = self.game.borrow();
            (game.board, nonzero_cells(&game))
        };
        self.overlay_box.set_visible(false);
        self.board.reset(board, spawns);
        self.update_scores();
    }

    fn continue_playing(&self) {
        self.game.borrow_mut().keep_playing = true;
        self.overlay_box.set_visible(false);
    }

    fn show_overlay(&self, title: &str, show_keep: bool) {
        self.overlay_title.set_text(title);
        self.keep_btn.set_visible(show_keep);
        self.overlay_box.set_visible(true);
    }

    /// Switch the whole app to another palette.
    fn apply_theme(&self, index: usize) {
        let palette = PALETTES[index];
        self.css_provider.load_from_string(&theme::css(palette));
        adw::StyleManager::default().set_color_scheme(palette.scheme);
        self.board.set_palette(palette);
        save_theme(index);
    }
}

fn build_ui(app: &adw::Application) {
    // Active mode: stylesheet, libadwaita scheme and board colors.
    let theme_index = load_theme();
    let palette = PALETTES[theme_index];
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_string(&theme::css(palette));
    let display = gdk::Display::default().expect("GDK display should be available");
    gtk::style_context_add_provider_for_display(
        &display,
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    gtk::IconTheme::for_display(&display).add_resource_path("/tiles/icons");
    adw::StyleManager::default().set_color_scheme(palette.scheme);

    // Header bar: app menu (color modes + about) on the right, like GNOME
    // core apps.
    let header = adw::HeaderBar::new();
    let menu = gio::Menu::new();
    let theme_section = gio::Menu::new();
    for palette in PALETTES {
        theme_section.append(
            Some(palette.label),
            Some(&format!("win.theme::{}", palette.name)),
        );
    }
    menu.append_section(Some("Color Mode"), &theme_section);
    let about_section = gio::Menu::new();
    about_section.append(Some("About Tiles"), Some("win.about"));
    menu.append_section(None, &about_section);
    let menu_btn = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .tooltip_text("Main menu")
        .build();
    header.pack_end(&menu_btn);

    // Title block on the left, score pills on the right — like play2048.co.
    let title_box = gtk::Box::new(Orientation::Vertical, 2);
    let title = Label::new(Some("Tiles"));
    title.add_css_class("game-title");
    title.set_halign(Align::Start);
    title.set_xalign(0.0);
    let subtitle = Label::new(Some("Join the numbers and get to the 2048 tile!"));
    subtitle.add_css_class("subtitle");
    subtitle.set_halign(Align::Start);
    subtitle.set_xalign(0.0);
    title_box.append(&title);
    title_box.append(&subtitle);
    title_box.set_hexpand(true);

    let (score_pill, score_label) = make_score_pill("Score");
    let (best_pill, best_label) = make_score_pill("Best");
    let scores = gtk::Box::new(Orientation::Horizontal, 10);
    scores.set_halign(Align::End);
    scores.set_valign(Align::Center);
    scores.append(&score_pill);
    scores.append(&best_pill);

    let title_row = gtk::Box::new(Orientation::Horizontal, 12);
    title_row.append(&title_box);
    title_row.append(&scores);

    // New Game on the left, hint on the right.
    let new_game_btn = Button::with_label("New Game");
    new_game_btn.add_css_class("new-game");
    new_game_btn.set_can_focus(false);
    let hint = Label::new(Some("Arrow keys, WASD or HJKL to move the tiles"));
    hint.add_css_class("hint");
    hint.set_halign(Align::End);
    hint.set_hexpand(true);
    hint.set_ellipsize(gtk::pango::EllipsizeMode::End);
    hint.set_margin_start(12);
    let action_row = gtk::Box::new(Orientation::Horizontal, 12);
    action_row.append(&new_game_btn);
    action_row.append(&hint);

    // The animated board with the win / game-over overlay on top.
    let board = BoardWidget::new(palette);

    let overlay_title = Label::new(None);
    overlay_title.add_css_class("overlay-title");
    let keep_btn = Button::with_label("Keep going");
    keep_btn.add_css_class("suggested-action");
    keep_btn.set_can_focus(false);
    let retry_btn = Button::with_label("New game");
    retry_btn.set_can_focus(false);
    let overlay_buttons = gtk::Box::new(Orientation::Horizontal, 10);
    overlay_buttons.set_halign(Align::Center);
    overlay_buttons.append(&keep_btn);
    overlay_buttons.append(&retry_btn);
    let overlay_box = gtk::Box::new(Orientation::Vertical, 14);
    overlay_box.add_css_class("overlay");
    overlay_box.set_halign(Align::Center);
    overlay_box.set_valign(Align::Center);
    overlay_box.append(&overlay_title);
    overlay_box.append(&overlay_buttons);
    overlay_box.set_visible(false);

    let board_overlay = Overlay::new();
    board_overlay.set_child(Some(&board));
    board_overlay.add_overlay(&overlay_box);

    // On-screen controls for mouse and touch.
    let dpad = gtk::Box::new(Orientation::Horizontal, 6);
    dpad.set_halign(Align::Center);
    let dpad_buttons: Vec<(Button, Direction)> = [
        ("←", Direction::Left),
        ("↑", Direction::Up),
        ("↓", Direction::Down),
        ("→", Direction::Right),
    ]
    .into_iter()
    .map(|(glyph, dir)| {
        let btn = Button::with_label(glyph);
        btn.add_css_class("flat");
        btn.set_can_focus(false);
        btn.set_size_request(52, 40);
        dpad.append(&btn);
        (btn, dir)
    })
    .collect();

    let page = gtk::Box::new(Orientation::Vertical, 16);
    page.set_margin_top(10);
    page.set_margin_bottom(16);
    page.set_margin_start(18);
    page.set_margin_end(18);
    page.set_valign(Align::Center);
    page.set_vexpand(true);
    page.append(&title_row);
    page.append(&action_row);
    page.append(&board_overlay);
    page.append(&dpad);

    let content = gtk::Box::new(Orientation::Vertical, 0);
    content.append(&header);
    content.append(&page);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Tiles")
        .default_width(500)
        .default_height(700)
        .content(&content)
        .build();
    window.set_icon_name(Some("io.github.hakergeniusz.Tiles"));

    let game = Game::new();
    let spawns = nonzero_cells(&game);
    let state = Rc::new(App {
        game: RefCell::new(game),
        best: Cell::new(load_best()),
        board,
        score_label,
        best_label,
        overlay_box,
        overlay_title,
        keep_btn: keep_btn.clone(),
        css_provider,
    });

    state.board.reset(state.game.borrow().board, spawns);

    new_game_btn.connect_clicked({
        let state = state.clone();
        move |_| state.new_game()
    });
    retry_btn.connect_clicked({
        let state = state.clone();
        move |_| state.new_game()
    });
    keep_btn.connect_clicked({
        let state = state.clone();
        move |_| state.continue_playing()
    });
    for (btn, dir) in dpad_buttons {
        let state = state.clone();
        btn.connect_clicked(move |_| state.clone().do_move(dir));
    }

    let controller = gtk::EventControllerKey::new();
    controller.connect_key_pressed({
        let state = state.clone();
        move |_, keyval, _, _| {
            let name = keyval.name().map(|n| n.to_string()).unwrap_or_default();
            let dir = match name.as_str() {
                "Left" | "a" | "A" | "h" | "H" => Some(Direction::Left),
                "Right" | "d" | "D" | "l" | "L" => Some(Direction::Right),
                "Up" | "w" | "W" | "k" | "K" => Some(Direction::Up),
                "Down" | "s" | "S" | "j" | "J" => Some(Direction::Down),
                _ => None,
            };
            match dir {
                Some(dir) => {
                    state.clone().do_move(dir);
                    glib::Propagation::Stop
                }
                None => glib::Propagation::Proceed,
            }
        }
    });
    window.add_controller(controller);

    // Mouse and touch swipes on the board: direction comes from the
    // dominant drag axis once the drag passes a short threshold.
    let swipe = gtk::GestureDrag::new();
    swipe.set_touch_only(false);
    swipe.set_button(0); // any mouse button
    swipe.connect_drag_end({
        let state = state.clone();
        move |_, dx, dy| {
            if (dx * dx + dy * dy).sqrt() < 24.0 {
                return;
            }
            let dir = if dx.abs() > dy.abs() {
                if dx > 0.0 {
                    Direction::Right
                } else {
                    Direction::Left
                }
            } else if dy > 0.0 {
                Direction::Down
            } else {
                Direction::Up
            };
            state.clone().do_move(dir);
        }
    });
    state.board.add_controller(swipe);

    // App menu actions: color modes and the About window.
    let theme_action = gio::SimpleAction::new_stateful(
        "theme",
        Some(glib::VariantTy::STRING),
        &PALETTES[theme_index].name.to_variant(),
    );
    {
        let state = state.clone();
        theme_action.connect_activate(move |action, param| {
            let name = param.and_then(|value| value.str()).unwrap_or("sepia");
            action.set_state(&name.to_variant());
            let index = PALETTES
                .iter()
                .position(|palette| palette.name == name)
                .unwrap_or(DEFAULT_THEME);
            state.apply_theme(index);
        });
    }
    window.add_action(&theme_action);

    // About window, laid out like GNOME core apps: logo, name, developer,
    // version badge, website / issue links, credits and legal pages.
    let about = adw::AboutWindow::builder()
        .application_name("Tiles")
        .application_icon("io.github.hakergeniusz.Tiles")
        .version(env!("CARGO_PKG_VERSION"))
        .developer_name("hakergeniusz")
        .website("https://github.com/hakergeniusz/tiles")
        .issue_url("https://github.com/hakergeniusz/tiles/issues")
        .copyright("© 2026 hakergeniusz")
        .license_type(gtk::License::MitX11)
        .transient_for(&window)
        .modal(true)
        .build();
    about.set_developers(&["hakergeniusz"]);
    let about_action = gio::SimpleAction::new("about", None);
    about_action.connect_activate(move |_, _| about.present());
    window.add_action(&about_action);

    window.present();
}

/// One score pill: a caption above a value label.
fn make_score_pill(caption: &str) -> (gtk::Box, Label) {
    let pill = gtk::Box::new(Orientation::Vertical, 0);
    pill.add_css_class("scorebox");
    let caption = Label::new(Some(caption));
    caption.add_css_class("caption");
    let value = Label::new(Some("0"));
    value.add_css_class("value");
    pill.append(&caption);
    pill.append(&value);
    (pill, value)
}

/// Cells occupied on a fresh board — used to animate their spawn.
fn nonzero_cells(game: &Game) -> Vec<(usize, usize)> {
    let mut cells = Vec::new();
    for row in 0..SIZE {
        for col in 0..SIZE {
            if game.board[row][col] != 0 {
                cells.push((row, col));
            }
        }
    }
    cells
}

/// Directory for the persisted best score and theme, `None` without a home.
fn data_dir() -> Option<PathBuf> {
    let base = match std::env::var_os("XDG_DATA_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(std::env::var_os("HOME")?).join(".local/share"),
    };
    Some(base.join("tiles"))
}

fn file_path(name: &str) -> Option<PathBuf> {
    Some(data_dir()?.join(name))
}

fn load_best() -> u32 {
    file_path("best")
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|content| content.trim().parse().ok())
        .unwrap_or(0)
}

fn save_best(best: u32) {
    let Some(path) = file_path("best") else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, best.to_string());
}

/// Index into [`PALETTES`] of the saved theme.
fn load_theme() -> usize {
    let name = file_path("theme")
        .and_then(|path| std::fs::read_to_string(path).ok())
        .unwrap_or_default();
    PALETTES
        .iter()
        .position(|palette| palette.name == name.trim())
        .unwrap_or(DEFAULT_THEME)
}

fn save_theme(index: usize) {
    let Some(path) = file_path("theme") else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, PALETTES[index].name);
}
