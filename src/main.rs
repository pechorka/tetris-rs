use raylib::prelude::*;
use raylib::prelude::KeyboardKey::*;

const BOARD_CELL_WIDTH: i32 = 11;
const BOARD_CELL_HEIGHT: i32 = BOARD_CELL_WIDTH * 2;
const BOARD_COLOR: Color = Color::RED;

const INITIAL_SCREEN_WIDTH: i32 = 1920;
const INITIAL_SCREEN_HEIGHT: i32 = 1080;

const EPS: f32 = 0.0001;

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
    bx1: i32,
    by1: i32,
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
            bx1: 0,
            by1: 0,
            bw: 0,
            bh: 0,
        };
        instance.recalculate();
        instance
    }

    fn update(&mut self, rl:&mut  RaylibHandle) {
        self.sw = rl.get_screen_width();
        self.sh = rl.get_screen_height();
        self.recalculate();
    }

    fn recalculate(&mut self) {
        let cell_size = 50;
        self.cw = self.sw / cell_size;
        self.ch = self.sh / (cell_size / (BOARD_CELL_HEIGHT/BOARD_CELL_WIDTH));

        self.bw = self.cw * BOARD_CELL_WIDTH;
        self.bh = self.ch * BOARD_CELL_HEIGHT;
        
        self.bx0 = self.sw / 2 - self.bw / 2;
        self.by0 = 100;
        self.bx1 = self.bx0 + self.bw;
        self.by1 = self.by0 + self.bh;
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
        Self{
            active_figure: Figure::random(&consts),
            placed_figures: Vec::new(),
            consts,
            score: 0,
        }
    }

    fn move_active_figure(&mut self, rl: &RaylibHandle) {
        let next_loc = self.active_figure.next_loc(&self.consts, rl);

        let active_with_next_loc = self.active_figure.copy_with_loc(next_loc);
        // TODO: sometimes figure goes below the board, fix it
        if float_gte(next_loc.y, self.consts.by1 as f32) || self.placed_figures.iter().any(|f| f.collides(&active_with_next_loc)) {
            self.placed_figures.push(self.active_figure);

            self.active_figure = Figure::random(&self.consts);
        } else {
            self.active_figure.update_loc(&self.consts, rl);
        }
    }

    fn draw_figures(&self, d: &mut RaylibDrawHandle) {
        self.active_figure.draw(&self.consts, d);

        for figure in &self.placed_figures {
            figure.draw(&self.consts, d);
        }
    }
    fn draw_board(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle_lines(self.consts.bx0, self.consts.by0, self.consts.bw, self.consts.bh, BOARD_COLOR);
        {
            let score_font_size: i32 = 20;
            let score_x: i32 = self.consts.bx0;
            let score_y: i32 = self.consts.by0 - score_font_size;
            d.draw_text(&format!("Score {score}", score = self.score), score_x, score_y, score_font_size, Color::BLACK);
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct FigureCommon {
    loc: Vector2,
    color: Color,
    animation_timer: f32,
}

impl FigureCommon {
    fn next_loc(&self, consts: &Constants, rl: &RaylibHandle) -> Vector2{
        let mut loc = self.loc;
        if is_one_of_keys_pressed(rl, &[KEY_A, KEY_LEFT]) && float_gte(self.loc.x - consts.cw as f32, consts.bx0 as f32) {
            loc.x -= consts.cw as f32
        } 
        if is_one_of_keys_pressed(rl, &[KEY_D, KEY_RIGHT]) && ((self.loc.x + consts.cw as f32) < consts.bx1 as f32) {
            loc.x += consts.cw as f32
        }
        if is_one_of_keys_down(rl, &[KEY_S, KEY_DOWN]) {
            loc.y += consts.ch as f32
        }

        if self.animation_timer <= 0.0 {
            loc.y += consts.ch as f32;
        }

        loc
    }

    fn update_loc(&mut self, consts: &Constants, rl: &RaylibHandle) {
        self.animation_timer -= rl.get_frame_time();
        self.loc = self.next_loc(consts, rl);
        if self.animation_timer <= 0.0 {
            self.animation_timer = 0.5;
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Figure {
    Square {  c: FigureCommon },
}

impl Figure {
    fn random(consts: &Constants) -> Self {
        match rand::random::<u8>() % 1 {
            0 => Self::Square {
                c: FigureCommon {
                    loc: Vector2::new((consts.bx0+consts.bw/2-consts.cw/2) as f32, consts.by0 as f32),
                    color: Color::GREEN,
                    animation_timer: 0.0,
                }
            },
            _ => panic!("Unknown figure type"),
        }
    }

    fn copy_with_loc(&self, loc: Vector2) -> Self {
        match self {
            Self::Square { c } => Self::Square {
                c: FigureCommon {
                    loc,
                    ..*c
                }
            },
        }
    }

    fn next_loc(&self, consts: &Constants, rl: &RaylibHandle) -> Vector2 {
        match self {
            Self::Square { c } => c.next_loc(consts, rl),
        }
    }

    fn update_loc(&mut self, consts: &Constants, rl: &RaylibHandle) {
        match self {
            Self::Square { c } => c.update_loc(consts, rl),
        }
    }

    fn draw(&self, consts: &Constants, d: &mut RaylibDrawHandle) {
        match self {
            Self::Square { c: FigureCommon{loc, color,..} } => {
                d.draw_rectangle_v(loc, Vector2::new(consts.cw as f32, consts.ch as f32), color);
                d.draw_rectangle_lines_ex(Rectangle::new(loc.x, loc.y, consts.cw as f32, consts.ch as f32), 2.0, Color::BLACK);
            }
        }
    }

    fn collides(&self, other: &Figure) -> bool {
        match (self, other) {
            (Self::Square { c: FigureCommon{loc: loc1, ..} }, Self::Square { c: FigureCommon{loc: loc2, ..} }) => {
                float_eq(loc1.x, loc2.x) && float_eq(loc1.y, loc2.y)
            }
        }
    }
}

fn float_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPS
}

fn float_gte(a: f32, b: f32) -> bool {
    a > b || float_eq(a, b)
}

fn is_one_of_keys_pressed(rl: &RaylibHandle, keys: &[KeyboardKey]) -> bool {
    keys.iter().any(|k| rl.is_key_pressed(*k))
}

fn is_one_of_keys_down(rl: &RaylibHandle, keys: &[KeyboardKey]) -> bool {
    keys.iter().any(|k| rl.is_key_down(*k))
}