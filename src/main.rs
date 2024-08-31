use raylib::prelude::KeyboardKey::*;
use raylib::prelude::*;

const BOARD_CELL_WIDTH: i32 = 11;
const BOARD_CELL_HEIGHT: i32 = BOARD_CELL_WIDTH * 2;
const BOARD_COLOR: Color = Color::RED;

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
        game.move_active_figure(&rl);

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);

        game.draw_board(&mut d);
        game.draw_figures(&mut d);
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

struct TetrisGame {
    active_figure: Figure,
    placed_figures: Vec<Figure>,

    consts: Constants,
    score: i32,
}

impl TetrisGame {
    fn new(consts: Constants) -> Self {
        Self {
            active_figure: Figure::random(),
            placed_figures: Vec::new(),
            consts,
            score: 0,
        }
    }

    fn move_active_figure(&mut self, rl: &RaylibHandle) {
        let mut next_loc = self.active_figure.get_loc();
        next_loc.x = self.active_figure.next_x(rl);

        if self.active_collides_at(&next_loc) {
            next_loc.x = self.active_figure.get_loc().x;
        }

        next_loc.y = self.active_figure.next_y(rl);

        if next_loc.y >= BOARD_CELL_HEIGHT {
            self.placed_figures.push(self.active_figure);
            self.active_figure = Figure::random();
            return;
        }

        if self.active_collides_at(&next_loc) {
            self.placed_figures.push(self.active_figure);
            self.active_figure = Figure::random();
        } else {
            self.active_figure.set_loc(next_loc);
            self.active_figure.update_timer(rl.get_frame_time());
        }
    }

    fn active_collides_at(&self, loc: &PositionOnBoard) -> bool {
        self.placed_figures
            .iter()
            .any(|f| f.collides_at(&self.active_figure, loc))
    }

    fn draw_figures(&self, d: &mut RaylibDrawHandle) {
        self.active_figure.draw(&self.consts, d);

        for figure in &self.placed_figures {
            figure.draw(&self.consts, d);
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

#[derive(Debug, Copy, Clone)]
struct PositionOnBoard {
    x: i32,
    y: i32,
}

impl PositionOnBoard {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Copy, Clone)]
struct FigureCommon {
    loc: PositionOnBoard,
    color: Color,
    animation_timer: f32,
}

impl FigureCommon {
    fn next_x(&self, rl: &RaylibHandle) -> i32 {
        let mut x = self.loc.x;
        if is_one_of_keys_pressed(rl, &[KEY_A, KEY_LEFT]) && x > 0 {
            x -= 1;
        }
        if is_one_of_keys_pressed(rl, &[KEY_D, KEY_RIGHT]) && x < BOARD_CELL_WIDTH - 1 {
            x += 1;
        }

        x
    }

    fn next_y(&self, rl: &RaylibHandle) -> i32 {
        let mut y = self.loc.y;
        if is_one_of_keys_down(rl, &[KEY_S, KEY_DOWN]) {
            y += 1;
        } else if self.animation_timer <= 0.0 {
            // else if to prevent double speed
            y += 1;
        }

        y
    }

    fn update_timer(&mut self, dt: f32) {
        if self.animation_timer <= 0.0 {
            self.animation_timer = 0.5;
        } else {
            self.animation_timer -= dt;
        }
    }

    fn set_loc(&mut self, loc: PositionOnBoard) {
        self.loc = loc;
    }
}

#[derive(Debug, Copy, Clone)]
enum Figure {
    Square { c: FigureCommon },
}

impl Figure {
    fn random() -> Self {
        match rand::random::<u8>() % 1 {
            0 => Self::Square {
                c: FigureCommon {
                    loc: PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 0),
                    color: Color::GREEN,
                    animation_timer: 0.0,
                },
            },
            _ => panic!("Unknown figure type"),
        }
    }

    fn get_loc(&self) -> PositionOnBoard {
        match self {
            Self::Square { c } => c.loc,
        }
    }

    fn set_loc(&mut self, loc: PositionOnBoard) {
        match self {
            Self::Square { c } => c.set_loc(loc),
        }
    }

    fn next_x(&self, rl: &RaylibHandle) -> i32 {
        match self {
            Self::Square { c } => c.next_x(rl),
        }
    }

    fn next_y(&self, rl: &RaylibHandle) -> i32 {
        match self {
            Self::Square { c } => c.next_y(rl),
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
                let x = consts.bx0 + loc.x * consts.cw;
                let y = consts.by0 + loc.y * consts.ch;
                d.draw_rectangle(x, y, consts.cw, consts.ch, *color);
                d.draw_rectangle_lines(x, y, consts.cw, consts.ch, Color::BLACK);
            }
        }
    }

    fn collides_at(&self, other: &Figure, other_next_loc: &PositionOnBoard) -> bool {
        match (self, other) {
            (
                Self::Square {
                    c: FigureCommon { loc, .. },
                },
                Self::Square { .. },
            ) => other_next_loc.x == loc.x && other_next_loc.y == loc.y,
        }
    }
}

fn is_one_of_keys_pressed(rl: &RaylibHandle, keys: &[KeyboardKey]) -> bool {
    keys.iter().any(|k| rl.is_key_pressed(*k))
}

fn is_one_of_keys_down(rl: &RaylibHandle, keys: &[KeyboardKey]) -> bool {
    keys.iter().any(|k| rl.is_key_down(*k))
}
