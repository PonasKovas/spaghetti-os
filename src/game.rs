use crate::VGA_BUFFER;

type PField = (usize, usize);

static mut STATE: GameState = GameState {
    direction: Direction::Right,
    last_direction: Direction::Right,
    world: [[Field::None; 39]; 22],
    score: 0,
    head_pos: (10, 15),
    tail_pos: (10, 13),
    dead: false,
};

pub struct GameState {
    direction: Direction,
    last_direction: Direction,
    // 2d array
    world: [[Field; 39]; 22],
    score: u16,
    head_pos: PField,
    tail_pos: PField,
    dead: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Field {
    None,
    Food,
    Snake(Option<PField>), // points to the parent snake part.
}

pub fn init_game() {
    unsafe {
        // init snake
        STATE.world[10][15] = Field::Snake(None); // head
        STATE.world[10][14] = Field::Snake(Some((10, 15)));
        STATE.world[10][13] = Field::Snake(Some((10, 14)));

        let (y, x) = get_random_food_pos();

        STATE.world[y][x] = Field::Food;
    }
}

pub fn set_direction(dir: Direction) {
    unsafe {
        if STATE.last_direction as u8 % 2 != dir as u8 % 2 {
            STATE.direction = dir;
        }
    }
}

fn get_random_food_pos() -> (usize, usize) {
    unsafe {
        for i in 0..22 {
            let y = (STATE.score as usize * 11327 + 123 + i) % 22;
            let mut x = crate::get_random_number() as usize % 39;
            for _ in 0..39 {
                if STATE.world[y][x] == Field::None {
                    return (y, x);
                }
                x = (x + 1) % 39;
            }
        }
        // the game is beaten and theres nowhere to spawn the food
        (0, 0)
    }
}

pub fn resume_game() {
    unsafe {
        if STATE.dead {
            // reset state
            STATE.direction = Direction::Right;
            STATE.last_direction = Direction::Right;
            STATE.world = [[Field::None; 39]; 22];
            STATE.score = 0;
            STATE.head_pos = (10, 15);
            STATE.tail_pos = (10, 13);
            STATE.dead = false;
            init_game();
        }
    }
}

pub fn tick() {
    unsafe {
        STATE.last_direction = STATE.direction;

        if STATE.dead {
            return;
        }

        // move the snake

        let current_head = (STATE.head_pos.0 + 22, STATE.head_pos.1 + 39);
        let new_head = match STATE.direction {
            Direction::Up => ((current_head.0 - 1) % 22, current_head.1 % 39),
            Direction::Down => ((current_head.0 + 1) % 22, current_head.1 % 39),
            Direction::Left => (current_head.0 % 22, (current_head.1 - 1) % 39),
            Direction::Right => (current_head.0 % 22, (current_head.1 + 1) % 39),
        };

        let ate_food = match STATE.world[new_head.0][new_head.1] {
            Field::Food => {
                STATE.score += 1;

                let (y, x) = get_random_food_pos();

                STATE.world[y][x] = Field::Food;

                true
            }
            Field::Snake(..) => {
                // oops! you died

                STATE.dead = true;
                draw_death();

                return;
            }
            _ => false,
        };

        STATE.world[new_head.0][new_head.1] = Field::Snake(None); // new head
        STATE.world[STATE.head_pos.0][STATE.head_pos.1] =
            Field::Snake(Some((new_head.0, new_head.1)));
        STATE.head_pos = new_head;
        if let Field::Snake(Some(p)) = STATE.world[STATE.tail_pos.0][STATE.tail_pos.1] {
            if !ate_food {
                STATE.world[STATE.tail_pos.0][STATE.tail_pos.1] = Field::None;
                STATE.tail_pos = p;
            }
        }

        // redraw
        draw();
    }
}

fn draw_death() {
    unsafe {
        for (i, &byte) in b" lmao u ded ".iter().enumerate() {
            *VGA_BUFFER.offset(12 * 160 + (34 + i as isize) * 2) = byte;
            *VGA_BUFFER.offset(12 * 160 + (34 + i as isize) * 2 + 1) = 0xf << 4;
        }
        for (i, &byte) in b" press entr to play ".iter().enumerate() {
            *VGA_BUFFER.offset(13 * 160 + (30 + i as isize) * 2) = byte;
            *VGA_BUFFER.offset(13 * 160 + (30 + i as isize) * 2 + 1) = 0xf << 4;
        }
    }
}

fn draw() {
    unsafe {
        // draws game
        for y in 0..22 {
            for (x, field) in STATE.world[y].iter().enumerate() {
                match field {
                    Field::None => {
                        *VGA_BUFFER.offset(((y + 1) * 160 + x * 4 + 1) as isize + 2) = 0x0;
                        *VGA_BUFFER.offset(((y + 1) * 160 + x * 4 + 3) as isize + 2) = 0x0;
                    }
                    Field::Food => {
                        *VGA_BUFFER.offset(((y + 1) * 160 + x * 4) as isize + 2) = 0xDB;
                        *VGA_BUFFER.offset(((y + 1) * 160 + x * 4 + 1) as isize + 2) = 0xf;
                        *VGA_BUFFER.offset(((y + 1) * 160 + x * 4 + 2) as isize + 2) = 0xDB;
                        *VGA_BUFFER.offset(((y + 1) * 160 + x * 4 + 3) as isize + 2) = 0xf;
                    }
                    Field::Snake(..) => {
                        *VGA_BUFFER.offset(((y + 1) * 160 + x * 4) as isize + 2) = 0xDB;
                        *VGA_BUFFER.offset(((y + 1) * 160 + x * 4 + 1) as isize + 2) = 0xa;
                        *VGA_BUFFER.offset(((y + 1) * 160 + x * 4 + 2) as isize + 2) = 0xDB;
                        *VGA_BUFFER.offset(((y + 1) * 160 + x * 4 + 3) as isize + 2) = 0xa;
                    }
                }
            }
        }
        draw_score();
    }
}

pub fn draw_border() {
    let top_bar: [u8; 80 * 2] = [
        0xC9, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf,
        0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf,
        0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf,
        0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf,
        0xCD, 0xf, 0xB9, 0xf, 's' as u8, 0xe, 'p' as u8, 0xe, 'a' as u8, 0xe, 'g' as u8, 0xe,
        'h' as u8, 0xe, 'e' as u8, 0xe, 't' as u8, 0xe, 't' as u8, 0xe, 'i' as u8, 0xe, ' ' as u8,
        0xe, 'O' as u8, 0xb, 'S' as u8, 0xb, 0xCC, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf,
        0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf,
        0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf,
        0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf,
        0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xBB, 0xf,
    ];

    unsafe { *VGA_BUFFER.cast() = top_bar };

    // sides
    for i in 1..24 {
        unsafe {
            // left
            *VGA_BUFFER.offset(i as isize * 160) = 0xBA;
            *VGA_BUFFER.offset(i as isize * 160 + 1) = 0xf;
            // right
            *VGA_BUFFER.offset(i as isize * 160 + 158) = 0xBA;
            *VGA_BUFFER.offset(i as isize * 160 + 159) = 0xf;
        }
    }

    let bottom_bar: [u8; 80 * 2] = [
        0xC8, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xB9, 0xf,
        '2' as u8, 0xd, '0' as u8, 0xd, '2' as u8, 0xd, '0' as u8, 0xd, 0xCC, 0xf, 0xCD, 0xf, 0xCD,
        0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD,
        0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD,
        0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD,
        0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD,
        0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD,
        0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD,
        0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xB9, 0xf, 'P' as u8, 0xd, 'o' as u8, 0xd, 'n' as u8,
        0xd, 'a' as u8, 0xd, 's' as u8, 0xd, 0xCC, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf, 0xCD, 0xf,
        0xCD, 0xf, 0xCD, 0xf, 0xBC, 0xf,
    ];

    unsafe { *VGA_BUFFER.offset(160 * 23).cast() = bottom_bar };
}

fn score_to_str(mut score: u16) -> [u8; 3] {
    let mut string = [0x30; 3];

    for i in 0..3 {
        string[2 - i] = (score % 10) as u8 + 48;
        score /= 10;
    }

    string
}

fn draw_score() {
    unsafe {
        let score_str = score_to_str(STATE.score);

        *VGA_BUFFER.offset(160 * 23 + 37 * 2) = 0xB9;
        *VGA_BUFFER.offset(160 * 23 + 37 * 2 + 1) = 0xf;
        *VGA_BUFFER.offset(160 * 23 + 41 * 2) = 0xCC;
        *VGA_BUFFER.offset(160 * 23 + 41 * 2 + 1) = 0xf;
        for (i, &byte) in score_str.iter().enumerate() {
            *VGA_BUFFER.offset(160 * 23 + (38 + i as isize) * 2) = byte;
            *VGA_BUFFER.offset(160 * 23 + (38 + i as isize) * 2 + 1) =
                (STATE.score % 0xe + 1) as u8;
        }
    }
}
