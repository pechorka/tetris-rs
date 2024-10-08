use raylib::prelude::KeyboardKey::*;
use raylib::prelude::*;

const BACKGROUND_COLOR: Color = Color::WHITE;

const BOARD_CELL_WIDTH: i32 = 10;
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
        let outside_of_board = loc
            .iter()
            .any(|pb| pb.y >= BOARD_CELL_HEIGHT || pb.x < 0 || pb.x >= BOARD_CELL_WIDTH);
        if outside_of_board {
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
        for loc in &self.active_figure.loc {
            let x = loc.x as usize;
            let y = loc.y as usize;
            self.placed_cells[y][x] = BoardCell::new(self.active_figure.color, *loc);
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
struct Figure {
    loc: Vec<PositionOnBoard>,
    color: Color,
    animation_timer: f32,
}

impl Figure {
    fn random() -> Self {
        match rand::random::<u8>() % 7 {
            0 => Self {
                // ##
                // ##
                loc: vec![
                    PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 0),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) + 1, 0),
                    PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 1),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) + 1, 1),
                ],
                color: Color::GOLD,
                animation_timer: 0.0,
            },
            1 => Self {
                // ####
                loc: vec![
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) - 1, 0),
                    PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 0),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) + 1, 0),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) + 2, 0),
                ],
                color: Color::TEAL,
                animation_timer: 0.0,
            },
            2 => Self {
                //  #
                // ###
                loc: vec![
                    PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 0),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) - 1, 1),
                    PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 1),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) + 1, 1),
                ],
                color: Color::NAVY,
                animation_timer: 0.0,
            },
            3 => Self {
                //   #
                // ###
                loc: vec![
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) + 1, 0),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) - 1, 1),
                    PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 1),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) + 1, 1),
                ],
                color: Color::YELLOWGREEN,
                animation_timer: 0.0,
            },
            4 => Self {
                // #
                // ###
                loc: vec![
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) - 1, 0),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) - 1, 1),
                    PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 1),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) + 1, 1),
                ],
                color: Color::DARKGREEN,
                animation_timer: 0.0,
            },
            5 => Self {
                // ##
                //  ##
                loc: vec![
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) - 1, 0),
                    PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 0),
                    PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 1),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) + 1, 1),
                ],
                color: Color::DARKMAGENTA,
                animation_timer: 0.0,
            },
            6 => Self {
                //  ##
                // ##
                loc: vec![
                    PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 0),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) + 1, 0),
                    PositionOnBoard::new((BOARD_CELL_WIDTH / 2) - 1, 1),
                    PositionOnBoard::new(BOARD_CELL_WIDTH / 2, 1),
                ],
                color: Color::BLUEVIOLET,
                animation_timer: 0.0,
            },
            _ => panic!("Unknown figure type"),
        }
    }

    fn move_h(&self, rl: &RaylibHandle) -> Vec<PositionOnBoard> {
        if is_one_of_keys_pressed(rl, &[KEY_A, KEY_LEFT]) {
            return self.loc.iter().map(|p| p.move_left()).collect();
        }
        if is_one_of_keys_pressed(rl, &[KEY_D, KEY_RIGHT]) {
            return self.loc.iter().map(|p| p.move_right()).collect();
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

    fn get_top_y(&self) -> i32 {
        self.loc
            .iter()
            .map(|pb| pb.y)
            .max()
            .unwrap_or(BOARD_CELL_HEIGHT)
    }

    fn draw(&self, consts: &Constants, d: &mut RaylibDrawHandle) {
        for cell_loc in &self.loc {
            draw_cell(cell_loc, consts, self.color, d)
        }
    }
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
