use raylib::prelude::*;
use raylib::prelude::KeyboardKey::*;

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
        game.draw_active_figure(&mut d);
    }
}


struct Constants {
    sw: i32,
    sh: i32,

    cw: i32,
    ch: i32,

    bx: i32,
    by: i32,
    bw: i32,
    bh: i32,
}

impl Constants {
    fn new(sw: i32, sh: i32) -> Self {
        let mut instance = Self {
            sw,
            sh,
            cw: 0,
            ch: 0,
            bx: 0,
            by: 0,
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
        
        self.bx = self.sw / 2 - self.bw / 2;
        self.by = 100;
    }
}

struct TetrisGame {
    active_figure: Figure,
    consts: Constants,
}

impl TetrisGame {
    fn new(consts: Constants) -> Self {
        return  Self{
            active_figure: Figure::random(&consts),
            consts,
        };
    }

    fn move_active_figure(&mut self, rl: &RaylibHandle) {
        self.active_figure.update_loc(&self.consts, rl);
    }

    fn draw_active_figure(&self, d: &mut RaylibDrawHandle) {
        self.active_figure.draw(&self.consts, d)
    }
    fn draw_board(&self, d: &mut RaylibDrawHandle) {
        d.draw_rectangle_lines(self.consts.bx, self.consts.by, self.consts.bw, self.consts.bh, BOARD_COLOR);
    }
}

struct FigureCommon {
    loc: Vector2,
    color: Color,
    animation_timer: f32,
}

impl FigureCommon {
    fn update_loc(&mut self, consts: &Constants, rl: &RaylibHandle) {
        if rl.is_key_pressed(KEY_A) {
            self.loc.x -= consts.cw as f32
        } else if rl.is_key_pressed(KEY_D) {
            self.loc.x += consts.cw as f32
        }

        if self.animation_timer > 0.0 {
            self.animation_timer -= rl.get_frame_time();
        } else {
            self.loc.y += consts.ch as f32;
            self.animation_timer = 0.5;
        }
    }
}

enum Figure {
    Square {  c: FigureCommon },
}

impl Figure {
    fn random(consts: &Constants) -> Self {
        match rand::random::<u8>() % 1 {
            0 => Self::Square {
                c: FigureCommon {
                    loc: Vector2::new((consts.bx+consts.bw/2-consts.cw/2) as f32, consts.by as f32),
                    color: Color::GREEN,
                    animation_timer: 0.0,
                }
            },
            _ => panic!("Unknown figure type"),
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
            }
        }
    }
}
