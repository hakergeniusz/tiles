//! Custom widget drawing the 2048 board with slide and pop animations.
//!
//! The whole board (background, cells, tiles, numbers) is painted with GTK
//! snapshot nodes. Moves are animated in two phases, like play2048.co:
//! a ~110 ms slide, then a ~190 ms pop for merged tiles and spawned tiles.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::OnceLock;

use adw::prelude::*;
use gtk::gdk::RGBA;
use gtk::glib;
use gtk::graphene;
use gtk::gsk;
use gtk::pango;
use gtk::subclass::prelude::*;
use gtk::{Align, Orientation};

use crate::game::{MoveEffect, SIZE};
use crate::theme::Palette;

/// Length of the slide phase, in milliseconds.
const SLIDE_MS: f64 = 110.0;
/// Length of the pop phase (merges and spawns), in milliseconds.
const POP_MS: f64 = 190.0;
/// The board is a fixed square, in pixels.
const BOARD_SIZE: i32 = 400;

/// Classic 2048 tile colors: `(value, background, foreground)`.
const TILE_COLORS: &[(u32, &str, &str)] = &[
    (2, "#eee4da", "#776e65"),
    (4, "#ede0c8", "#776e65"),
    (8, "#f2b179", "#f9f6f2"),
    (16, "#f59563", "#f9f6f2"),
    (32, "#f67c5f", "#f9f6f2"),
    (64, "#f65e3b", "#f9f6f2"),
    (128, "#edcf72", "#f9f6f2"),
    (256, "#edcc61", "#f9f6f2"),
    (512, "#edc850", "#f9f6f2"),
    (1024, "#edc53f", "#f9f6f2"),
    (2048, "#edc22e", "#f9f6f2"),
];

fn parse_color(color: &str) -> RGBA {
    RGBA::parse(color).expect("hard-coded color should parse")
}

/// Parsed tile colors, computed once.
fn tile_styles() -> &'static [(u32, RGBA, RGBA)] {
    static STYLES: OnceLock<Vec<(u32, RGBA, RGBA)>> = OnceLock::new();
    STYLES.get_or_init(|| {
        TILE_COLORS
            .iter()
            .map(|&(value, bg, fg)| (value, parse_color(bg), parse_color(fg)))
            .collect()
    })
}

fn tile_style(value: u32) -> (RGBA, RGBA) {
    for &(v, bg, fg) in tile_styles() {
        if v == value {
            return (bg, fg);
        }
    }
    // 4096 and beyond: the dark super tile.
    (parse_color("#3c3a32"), parse_color("#f9f6f2"))
}

/// Ease-out cubic, for slides and spawn growth.
fn ease(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

fn rect(x: f64, y: f64, w: f64, h: f64) -> graphene::Rect {
    graphene::Rect::new(x as f32, y as f32, w as f32, h as f32)
}

fn rounded(x: f64, y: f64, w: f64, h: f64, radius: f64) -> gsk::RoundedRect {
    let r = radius as f32;
    gsk::RoundedRect::new(
        rect(x, y, w, h),
        graphene::Size::new(r, r),
        graphene::Size::new(r, r),
        graphene::Size::new(r, r),
        graphene::Size::new(r, r),
    )
}

fn with_alpha(color: RGBA, alpha: f64) -> RGBA {
    RGBA::new(
        color.red(),
        color.green(),
        color.blue(),
        color.alpha() * alpha as f32,
    )
}

/// Top-left corner of cell `(row, col)` on the board.
fn cell_origin(ox: f64, oy: f64, pad: f64, step: f64, row: usize, col: usize) -> (f64, f64) {
    (ox + pad + col as f64 * step, oy + pad + row as f64 * step)
}

/// Relative font size for a tile value.
fn font_factor(value: u32) -> f64 {
    match value.to_string().len() {
        1 | 2 => 0.45,
        3 => 0.36,
        4 => 0.29,
        _ => 0.24,
    }
}

mod imp {
    use super::*;

    /// One tile sliding between two cells.
    pub struct SlideTile {
        pub value: u32,
        pub from: (usize, usize),
        pub to: (usize, usize),
    }

    /// A running animation: slide the tiles, then pop merges and spawns.
    pub struct Anim {
        /// Frame-clock time (µs) the animation started, set lazily.
        pub start: Cell<Option<i64>>,
        pub slide_tiles: Vec<SlideTile>,
        pub merged: Vec<(usize, usize)>,
        pub spawns: Vec<(usize, usize)>,
        pub old_board: [[u32; SIZE]; SIZE],
        pub final_board: [[u32; SIZE]; SIZE],
    }

    /// Board background colors, from the active palette.
    #[derive(Clone, Copy)]
    pub struct BoardColors {
        pub board_bg: RGBA,
        pub empty_tile: RGBA,
    }

    pub struct BoardWidget {
        pub colors: RefCell<BoardColors>,
        pub board: RefCell<[[u32; SIZE]; SIZE]>,
        pub anim: RefCell<Option<Anim>>,
        pub tick_id: RefCell<Option<gtk::TickCallbackId>>,
        pub layouts: RefCell<HashMap<u32, pango::Layout>>,
        pub cell_size: Cell<f64>,
    }

    impl Default for BoardWidget {
        fn default() -> Self {
            Self {
                colors: RefCell::new(BoardColors {
                    board_bg: parse_color("#bbada0"),
                    empty_tile: parse_color("rgba(238, 228, 218, 0.35)"),
                }),
                board: RefCell::new([[0; SIZE]; SIZE]),
                anim: RefCell::new(None),
                tick_id: RefCell::new(None),
                layouts: RefCell::new(HashMap::new()),
                cell_size: Cell::new(0.0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BoardWidget {
        const NAME: &'static str = "Gtk2048Board";
        type Type = super::BoardWidget;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for BoardWidget {}

    impl WidgetImpl for BoardWidget {
        fn measure(&self, _orientation: Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            (BOARD_SIZE, BOARD_SIZE, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            self.draw_board(snapshot);
        }
    }

    impl BoardWidget {
        /// Draw the board background, the empty cells and the tiles —
        /// interpolating positions and scales while an animation runs.
        fn draw_board(&self, snapshot: &gtk::Snapshot) {
            let width = self.obj().width() as f64;
            let height = self.obj().height() as f64;
            let size = width.min(height);
            if size < 32.0 {
                return;
            }
            let gap = (size / 40.0).round().max(6.0);
            let cell = (size - 5.0 * gap) / 4.0;
            let step = cell + gap;
            let ox = (width - size) / 2.0;
            let oy = (height - size) / 2.0;

            if self.cell_size.get() != cell {
                self.cell_size.set(cell);
                self.layouts.borrow_mut().clear();
            }
            let colors = *self.colors.borrow();

            // Board background and empty cells.
            self.fill_rounded(
                snapshot,
                ox,
                oy,
                size,
                size,
                size / 24.0,
                colors.board_bg,
                1.0,
            );
            for row in 0..SIZE {
                for col in 0..SIZE {
                    let (x, y) = cell_origin(ox, oy, gap, step, row, col);
                    self.fill_rounded(
                        snapshot,
                        x,
                        y,
                        cell,
                        cell,
                        cell * 0.12,
                        colors.empty_tile,
                        1.0,
                    );
                }
            }

            let now = self.obj().frame_clock().map(|clock| clock.frame_time());
            let anim = self.anim.borrow();
            let (t_slide, t_pop) = match (&*anim, now) {
                (Some(a), Some(now)) => {
                    let start = a.start.get().unwrap_or(now);
                    let elapsed = (now - start).max(0) as f64 / 1000.0;
                    let slide = if a.slide_tiles.is_empty() {
                        1.0
                    } else {
                        (elapsed / SLIDE_MS).clamp(0.0, 1.0)
                    };
                    let pop = ((elapsed - SLIDE_MS) / POP_MS).clamp(0.0, 1.0);
                    (slide, pop)
                }
                _ => (1.0, 1.0),
            };

            if t_slide < 1.0 {
                // Slide phase: the old board, with the moved tiles interpolated.
                let a = anim.as_ref().unwrap();
                for row in 0..SIZE {
                    for col in 0..SIZE {
                        let value = a.old_board[row][col];
                        if value == 0 || a.slide_tiles.iter().any(|t| t.from == (row, col)) {
                            continue;
                        }
                        let (x, y) = cell_origin(ox, oy, gap, step, row, col);
                        self.draw_tile(snapshot, x, y, cell, value, 1.0, 1.0);
                    }
                }
                for tile in &a.slide_tiles {
                    let t = ease(t_slide);
                    let (fx, fy) = cell_origin(ox, oy, gap, step, tile.from.0, tile.from.1);
                    let (tx, ty) = cell_origin(ox, oy, gap, step, tile.to.0, tile.to.1);
                    let x = fx + (tx - fx) * t;
                    let y = fy + (ty - fy) * t;
                    self.draw_tile(snapshot, x, y, cell, tile.value, 1.0, 1.0);
                }
            } else {
                // Settled or pop phase: the final board.
                let board = match anim.as_ref() {
                    Some(a) => a.final_board,
                    None => *self.board.borrow(),
                };
                for row in 0..SIZE {
                    for col in 0..SIZE {
                        let value = board[row][col];
                        if value == 0 {
                            continue;
                        }
                        let (x, y) = cell_origin(ox, oy, gap, step, row, col);
                        let mut scale = 1.0;
                        let mut alpha = 1.0;
                        if let Some(a) = anim.as_ref() {
                            if a.merged.contains(&(row, col)) {
                                // Merged tiles pulse up and settle back.
                                scale = 1.0 + 0.16 * (PI * t_pop).sin();
                            } else if a.spawns.contains(&(row, col)) {
                                // Spawned tiles grow in and fade up.
                                let t = ease(t_pop);
                                scale = t.max(0.05);
                                alpha = t;
                            }
                        }
                        self.draw_tile(snapshot, x, y, cell, value, scale, alpha);
                    }
                }
            }
        }

        /// One tile: rounded background plus a centred, scaled number.
        fn draw_tile(
            &self,
            snapshot: &gtk::Snapshot,
            x: f64,
            y: f64,
            cell: f64,
            value: u32,
            scale: f64,
            alpha: f64,
        ) {
            let (bg, fg) = tile_style(value);
            let size = cell * scale;
            let off = (cell - size) / 2.0;

            snapshot.push_rounded_clip(&rounded(x + off, y + off, size, size, cell * 0.12 * scale));
            snapshot.append_color(&with_alpha(bg, alpha), &rect(x + off, y + off, size, size));

            let layout = self.layout_for(value, cell);
            snapshot.save();
            // Scale the number around the tile centre.
            snapshot.translate(&graphene::Point::new(
                (x + cell / 2.0) as f32,
                (y + cell / 2.0) as f32,
            ));
            snapshot.scale(scale as f32, scale as f32);
            snapshot.translate(&graphene::Point::new(
                -(x + cell / 2.0) as f32,
                -(y + cell / 2.0) as f32,
            ));
            let (layout_w, layout_h) = layout.pixel_size();
            snapshot.translate(&graphene::Point::new(
                (x + (cell - layout_w as f64) / 2.0) as f32,
                (y + (cell - layout_h as f64) / 2.0) as f32,
            ));
            snapshot.append_layout(&layout, &with_alpha(fg, alpha));
            snapshot.restore();
            snapshot.pop();
        }

        fn fill_rounded(
            &self,
            snapshot: &gtk::Snapshot,
            x: f64,
            y: f64,
            w: f64,
            h: f64,
            radius: f64,
            color: RGBA,
            alpha: f64,
        ) {
            snapshot.push_rounded_clip(&rounded(x, y, w, h, radius));
            snapshot.append_color(&with_alpha(color, alpha), &rect(x, y, w, h));
            snapshot.pop();
        }

        /// A cached pango layout for a tile value.
        fn layout_for(&self, value: u32, cell: f64) -> pango::Layout {
            if let Some(layout) = self.layouts.borrow().get(&value) {
                return layout.clone();
            }
            let layout = self.obj().create_pango_layout(None);
            layout.set_text(&value.to_string());
            let mut font = pango::FontDescription::new();
            font.set_family("Sans");
            font.set_weight(pango::Weight::Bold);
            font.set_absolute_size(cell * font_factor(value) * pango::SCALE as f64);
            layout.set_font_description(Some(&font));
            self.layouts.borrow_mut().insert(value, layout.clone());
            layout
        }

        /// Begin animating; a frame-clock callback drives the redraws.
        pub fn start_anim(&self, anim: Anim) {
            *self.anim.borrow_mut() = Some(anim);
            if let Some(id) = self.tick_id.borrow_mut().take() {
                id.remove();
            }

            let widget = self.obj().clone();
            let tick_id = widget.add_tick_callback({
                // Move a clone into the closure; the receiver borrow above
                // must not be moved out of.
                let widget = widget.clone();
                move |_, clock| {
                    let imp = widget.imp();
                    let done = {
                        let mut slot = imp.anim.borrow_mut();
                        match slot.as_mut() {
                            None => true,
                            Some(a) => {
                                let now = clock.frame_time();
                                if a.start.get().is_none() {
                                    a.start.set(Some(now));
                                }
                                let elapsed = (now - a.start.get().unwrap()).max(0) as f64 / 1000.0;
                                elapsed >= SLIDE_MS + POP_MS
                            }
                        }
                    };
                    if done {
                        if let Some(a) = imp.anim.borrow_mut().take() {
                            *imp.board.borrow_mut() = a.final_board;
                        }
                        *imp.tick_id.borrow_mut() = None;
                        widget.queue_draw();
                        glib::ControlFlow::Break
                    } else {
                        widget.queue_draw();
                        glib::ControlFlow::Continue
                    }
                }
            });
            *self.tick_id.borrow_mut() = Some(tick_id);
        }
    }
}

glib::wrapper! {
    pub struct BoardWidget(ObjectSubclass<imp::BoardWidget>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl BoardWidget {
    pub fn new(palette: &Palette) -> Self {
        let widget: Self = glib::Object::builder().build();
        widget.set_palette(palette);
        widget.set_size_request(BOARD_SIZE, BOARD_SIZE);
        widget.set_halign(Align::Center);
        widget.set_valign(Align::Center);
        widget
    }

    /// Apply the board colors of a palette.
    pub fn set_palette(&self, palette: &Palette) {
        let colors = imp::BoardColors {
            board_bg: parse_color(palette.board_bg),
            empty_tile: parse_color(palette.empty_tile),
        };
        *self.imp().colors.borrow_mut() = colors;
        self.queue_draw();
    }

    /// Show a fresh board, popping in the given tiles.
    pub fn reset(&self, board: [[u32; SIZE]; SIZE], spawns: Vec<(usize, usize)>) {
        self.finish_animation();
        let imp = self.imp();
        *imp.board.borrow_mut() = board;
        if !spawns.is_empty() {
            imp.start_anim(imp::Anim {
                start: Cell::new(self.frame_clock().map(|clock| clock.frame_time())),
                slide_tiles: Vec::new(),
                merged: Vec::new(),
                spawns,
                old_board: board,
                final_board: board,
            });
        }
        self.queue_draw();
    }

    /// Animate a move: slide according to `effect`, then pop the merges and
    /// the spawned tile. `final_board` is the board after both.
    pub fn play_move(
        &self,
        effect: &MoveEffect,
        spawn: Option<(usize, usize)>,
        final_board: [[u32; SIZE]; SIZE],
    ) {
        self.finish_animation();
        let imp = self.imp();
        let old_board = *imp.board.borrow();
        let slide_tiles = effect
            .moves
            .iter()
            .map(|&(from, to)| imp::SlideTile {
                value: old_board[from.0][from.1],
                from,
                to,
            })
            .collect();
        imp.start_anim(imp::Anim {
            start: Cell::new(self.frame_clock().map(|clock| clock.frame_time())),
            slide_tiles,
            merged: effect.merged.clone(),
            spawns: spawn.map(|cell| vec![cell]).unwrap_or_default(),
            old_board,
            final_board,
        });
        self.queue_draw();
    }

    /// Jump to the end state of the current animation, if any.
    pub fn finish_animation(&self) {
        let imp = self.imp();
        if let Some(anim) = imp.anim.borrow_mut().take() {
            *imp.board.borrow_mut() = anim.final_board;
        }
        if let Some(id) = imp.tick_id.borrow_mut().take() {
            id.remove();
        }
        self.queue_draw();
    }

    /// Whether a move animation is currently running.
    pub fn is_animating(&self) -> bool {
        self.imp().anim.borrow().is_some()
    }
}
