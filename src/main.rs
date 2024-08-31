use std::fmt::Pointer;

use raylib::prelude::KeyboardKey::*;
use raylib::prelude::*;

const BACKGROUND_COLOR: Color = Color::WHITE;

const BOARD_CELL_WIDTH: i32 = 11;
const BOARD_CELL_HEIGHT: i32 = BOARD_CELL_WIDTH * 2;
const BOARD_COLOR: Color = Color::RED;
const BOARD_BACKGROUND_COLOR: Color = BACKGROUND_COLOR;

const INITIAL_SCREEN_WIDTH: i32 = 1920;
const INITIAL_SCREEN_HEIGHT: i32 = 1080;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(INITIAL_SCREEN_WIDTH, INITIAL_SCREEN_HEIGHT)
        .title("Hello, World")
        .build();

    rl.set_target_fps(60);

    let mut game = TetrisGame::new(Constants::new(INITIAL_SCREEN_WIDTH, INITIAL_SCREEN_HEIGHT));

    while !rl.window_should_close() {
        game.consts.update(&mut rl);
        if !game.game_over {
            game.move_active_figure(&rl);
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(BACKGROUND_COLOR);

        game.draw_board(&mut d);
        game.draw_figures(&mut d);

        if game.game_over {
            d.draw_text("Game Over", 100, 100, 50, Color::BLACK);
        }
    }
}

struct Constants {
    sw: i32,
    sh: i32,

    cw: i32,
    ch: i32,

    bw: i32,
    bh: i32,
    bx0: i32,
    by0: i32,
}

impl Constants {
    fn new(sw: i32, sh: i32) -> Self {
        let mut instance = Self {
            sw,
            sh,
            cw: 0,
            ch: 0,
            bx0: 0,
            by0: 0,
            bw: 0,
            bh: 0,
        };
        instance.recalculate();
        instance
    }

    fn update(&mut self, rl: &mut RaylibHandle) {
        self.sw = rl.get_screen_width();
        self.sh = rl.get_screen_height();
        self.recalculate();
    }

    fn recalculate(&mut self) {
        let cell_size = 50;
        self.cw = self.sw / cell_size;
        self.ch = self.sh / (cell_size / (BOARD_CELL_HEIGHT / BOARD_CELL_WIDTH));

        self.bw = self.cw * BOARD_CELL_WIDTH;
        self.bh = self.ch * BOARD_CELL_HEIGHT;

        self.bx0 = self.sw / 2 - self.bw / 2;
        self.by0 = 100;
    }
}

#[derive(Clone, Copy, Debug)]
struct BoardCell {
    color: Color,
    loc: PositionOnBoard,
    filled: bool,
}

impl BoardCell {
    fn new(color: Color, loc: PositionOnBoard) -> Self {
        return Self {
            color,
            loc,
            filled: true,
        };
    }

    fn zero() -> Self {
        return Self {
            color: BOARD_BACKGROUND_COLOR,
            loc: PositionOnBoard::new(0, 0),
            filled: false,
        };
    }
}

struct TetrisGame {
    active_figure: Figure,
    placed_cells: Vec<Vec<BoardCell>>,

    consts: Constants,
    score: i32,

    game_over: bool,
}

impl TetrisGame {
    fn new(consts: Constants) -> Self {
        Self {
            active_figure: Figure::random(),
            placed_cells: vec![
                vec![BoardCell::zero(); BOARD_CELL_WIDTH as usize];
                BOARD_CELL_HEIGHT as usize
            ],
            consts,
            score: 0,
            game_over: false,
        }
    }

    fn move_active_figure(&mut self, rl: &RaylibHandle) {
        let mut next_loc = self.active_figure.move_h(rl);

        if !self.collides_at(&next_loc) {
            self.active_figure.set_loc(next_loc);
        }

        next_loc = self.active_figure.move_v(rl);

        if self.collides_at(&next_loc) {
            if cell_to_screen_y(self.active_figure.get_top_y(), &self.consts) <= self.consts.by0 {
                self.game_over = true;
                return;
            }
            self.place_active_figure();
            self.clear_lines();
            self.active_figure = Figure::random();
        } else {
            self.active_figure.set_loc(next_loc);
            self.active_figure.update_timer(rl.get_frame_time());
        }
    }

    fn collides_at(&self, loc: &Vec<PositionOnBoard>) -> bool {
        let bellow_board = loc.iter().any(|pb| pb.y >= BOARD_CELL_HEIGHT);
        if bellow_board {
            return true;
        }
        for row in &self.placed_cells {
            for cell in row {
                if loc.contains(&cell.loc) {
                    return true;
                }
            }
        }
        return false;
    }

    fn place_active_figure(&mut self) {
        for loc in self.active_figure.get_loc() {
            let x = loc.x as usize;
            let y = loc.y as usize;
            self.placed_cells[y][x] = BoardCell::new(self.active_figure.get_color(), loc);
        }
    }

    fn clear_lines(&mut self) {
        let mut lines_cleared = 0;

        // First, identify and clear full lines
        for y in (0..BOARD_CELL_HEIGHT as usize).rev() {
            if self.placed_cells[y].iter().all(|bc| bc.filled) {
                // Clear the line
                self.placed_cells[y] = vec![BoardCell::zero(); BOARD_CELL_WIDTH as usize];
                lines_cleared += 1;
            }
        }

        if lines_cleared == 0 {
            return;
        }

        // Update score
        self.score += lines_cleared;

        // Move remaining blocks down
        for y in (0..BOARD_CELL_HEIGHT as usize).rev() {
            if self.placed_cells[y].iter().all(|bc| !bc.filled) {
                // Find the next non-empty row above
                if let Some(next_filled_row) = (0..y)
                    .rev()
                    .find(|&i| self.placed_cells[i].iter().any(|bc| bc.filled))
                {
                    // Move the non-empty row down
                    self.placed_cells[y] = self.placed_cells[next_filled_row].clone();
                    self.placed_cells[next_filled_row] =
                        vec![BoardCell::zero(); BOARD_CELL_WIDTH as usize];
                } else {
                    // If no more filled rows above, we're done
                    break;
                }
            }
        }

        // Update positions of remaining cells
        for y in 0..BOARD_CELL_HEIGHT as usize {
            for x in 0..BOARD_CELL_WIDTH as usize {
                if self.placed_cells[y][x].filled {
                    self.placed_cells[y][x].loc = PositionOnBoard::new(x as i32, y as i32);
                }
            }
        }
    }

    fn draw_figures(&self, d: &mut RaylibDrawHandle) {
        self.active_figure.draw(&self.consts, d);

        for row in &self.placed_cells {
            for cell in row {
                if cell.filled {
                    draw_cell(&cell.loc, &self.consts, cell.color, d)
                }
            }
        }
    }
    fn draw_board(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle_lines(
            self.consts.bx0,
            self.consts.by0,
            self.consts.bw,
            self.consts.bh,
            BOARD_COLOR,
        );
        {
            let score_font_size: i32 = 20;
            let score_x: i32 = self.consts.bx0;
            let score_y: i32 = self.consts.by0 - score_font_size;
            d.draw_text(
                &format!("Score {score}", score = self.score),
                score_x,
                score_y,
                score_font_size,
                Color::BLACK,
            );
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct PositionOnBoard {
    x: i32,
    y: i32,
}

impl PositionOnBoard {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn move_left(&self) -> Self {
        return Self {
            x: self.x - 1,
            y: self.y,
        };
    }

    fn move_right(&self) -> Self {
        return Self {
            x: self.x + 1,
            y: self.y,
        };
    }

    fn move_down(&self) -> Self {
        return Self {
            x: self.x,
            y: self.y + 1,
        };
    }
}

#[derive(Debug, Clone)]
struct FigureCommon {
    loc: Vec<PositionOnBoard>,
    color: Color,
    animation_timer: f32,
}

impl FigureCommon {
    fn move_h(&self, rl: &RaylibHandle) -> Vec<PositionOnBoard> {
        if is_one_of_keys_pressed(rl, &[KEY_A, KEY_LEFT]) {
            return self
                .loc
                .iter()
                .map(|p| if p.x > 0 { p.move_left() } else { *p })
                .collect();
        }
        if is_one_of_keys_pressed(rl, &[KEY_D, KEY_RIGHT]) {
            return self
                .loc
                .iter()
                .map(|p| {
                    if p.x < BOARD_CELL_WIDTH - 1 {
                        p.move_right()
                    } else {
                        *p
                    }
                })
                .collect();
        }

        self.loc.clone()
    }

    fn move_v(&self, rl: &RaylibHandle) -> Vec<PositionOnBoard> {
        if is_one_of_keys_down(rl, &[KEY_S, KEY_DOWN]) || self.animation_timer <= 0.0 {
            return self.loc.iter().map(|p| p.move_down()).collect();
        }

        self.loc.clone()
    }

    fn update_timer(&mut self, dt: f32) {
        if self.animation_timer <= 0.0 {
            self.animation_timer = 0.5;
        } else {
            self.animation_timer -= dt;
        }
    }

    fn set_loc(&mut self, loc: Vec<PositionOnBoard>) {
        self.loc = loc;
    }
}

#[derive(Debug, Clone)]
enum Figure {
    Square { c: FigureCommon },
}

impl Figure {
    fn random() -> Self {
        match rand::random::<u8>() % 1 {
            0 => Self::Square {
                c: FigureCommon {
                    loc: vec![PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 0)],
                    color: Color::GREEN,
                    animation_timer: 0.0,
                },
            },
            _ => panic!("Unknown figure type"),
        }
    }

    fn get_color(&self) -> Color {
        match self {
            Self::Square { c } => c.color,
        }
    }

    fn get_loc(&self) -> Vec<PositionOnBoard> {
        match self {
            Self::Square { c } => c.loc.clone(),
        }
    }

    fn get_top_y(&self) -> i32 {
        match self {
            Self::Square { c } => top_y(&c.loc),
        }
    }

    fn set_loc(&mut self, loc: Vec<PositionOnBoard>) {
        match self {
            Self::Square { c } => c.set_loc(loc),
        }
    }

    fn move_v(&self, rl: &RaylibHandle) -> Vec<PositionOnBoard> {
        match self {
            Self::Square { c } => c.move_v(rl),
        }
    }

    fn move_h(&self, rl: &RaylibHandle) -> Vec<PositionOnBoard> {
        match self {
            Self::Square { c } => c.move_h(rl),
        }
    }

    fn update_timer(&mut self, dt: f32) {
        match self {
            Self::Square { c } => c.update_timer(dt),
        }
    }

    fn draw(&self, consts: &Constants, d: &mut RaylibDrawHandle) {
        match self {
            Self::Square {
                c: FigureCommon { loc, color, .. },
            } => {
                for cell_loc in loc {
                    draw_cell(cell_loc, consts, *color, d)
                }
            }
        }
    }
}

fn top_y(loc: &Vec<PositionOnBoard>) -> i32 {
    return loc.iter().map(|p| p.y).max().unwrap_or(BOARD_CELL_HEIGHT);
}

fn draw_cell(loc: &PositionOnBoard, consts: &Constants, color: Color, d: &mut RaylibDrawHandle) {
    let x = cell_to_screen_x(loc.x, consts);
    let y = cell_to_screen_y(loc.y, consts);
    d.draw_rectangle(x, y, consts.cw, consts.ch, color);
    d.draw_rectangle_lines(x, y, consts.cw, consts.ch, Color::BLACK);
}

fn cell_to_screen_x(cell_x: i32, consts: &Constants) -> i32 {
    consts.bx0 + cell_x * consts.cw
}

fn cell_to_screen_y(cell_y: i32, consts: &Constants) -> i32 {
    consts.by0 + cell_y * consts.ch
}

fn is_one_of_keys_pressed(rl: &RaylibHandle, keys: &[KeyboardKey]) -> bool {
    keys.iter().any(|k| rl.is_key_pressed(*k))
}

fn is_one_of_keys_down(rl: &RaylibHandle, keys: &[KeyboardKey]) -> bool {
    keys.iter().any(|k| rl.is_key_down(*k))
}
