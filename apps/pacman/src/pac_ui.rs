use numworks_utils::{
    eadk::{
        display::{draw_string, push_rect_uniform, wait_for_vblank, SCREEN_HEIGHT, SCREEN_WIDTH},
        timing::msleep,
        Color, Point, Rect,
    },
    graphical::tiling::Tileset,
    include_bytes_align_as,
    utils::{get_string_pixel_size, string_from_u16, string_from_u32, SMALL_CHAR_HEIGHT},
};

use crate::{
    game::{Grid, Space, GRID_HEIGHT, GRID_WIDTH, MAZE_BYTES, TILE_SIZE, X_GRID_OFFSET},
    ghost::{Ghost, GhostType, HouseState, MovementMode},
    moveable::Direction,
    player::Player,
};

const WALL_IMAGES_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/walls.nppm");
const SPRITES_IMAGES_BYTES: &[u8] = include_bytes_align_as!(Color, "./data/sprites.nppm");
const GRID_SIZE: usize = (GRID_WIDTH as usize) * (GRID_HEIGHT as usize);

/// The maze is printed a first time but also every frame around each Moveable object.
/// That being heavy in calculation, we cache the tileset positions corresponding to each grid positions.
static mut MAZE_TILE_CACHE: [Point; GRID_SIZE] = [Point { x: 12, y: 2 }; GRID_SIZE];
static mut MAZE_TILE_CACHE_READY: bool = false;
fn ensure_maze_tile_cache() {
    unsafe {
        if !MAZE_TILE_CACHE_READY {
            let mut index = 0;
            for line in MAZE_BYTES
                .lines()
                .filter(|s| !s.is_empty())
                .take(GRID_HEIGHT as usize)
            {
                for c in line.chars().take(GRID_WIDTH as usize) {
                    MAZE_TILE_CACHE[index] = get_tile_position(c).unwrap_or(Point { x: 12, y: 2 });
                    index += 1;
                }
            }
            MAZE_TILE_CACHE_READY = true;
        }
    }
}

/// I'm using two different Tilesets because one is using 8 pixels wide tiles and the second 16 pixels wide.
pub static TILESET_WALLS: Tileset = Tileset::new(TILE_SIZE, 16, WALL_IMAGES_BYTES);
pub static TILESET_SPRITES: Tileset = Tileset::new(TILE_SIZE * 2, 8, SPRITES_IMAGES_BYTES);

/// Gives the absolute pixel position from the given position on the grid.
const fn abs_from_pos(pos: Point) -> Point {
    Point {
        x: pos.x * TILE_SIZE + X_GRID_OFFSET,
        y: pos.y * TILE_SIZE,
    }
}

/// Draws Pac-Man.
pub fn draw_player(pos: Point, steps: u8, dir: &Direction, frames: u32, wrapping: bool) {
    let np = abs_from_pos(pos);
    let offset = match dir {
        Direction::Up | Direction::Down => 0,
        Direction::Right | Direction::Left => 1,
    };
    let p = Point {
        x: (np.x as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().0) as u16,
        y: (np.y as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().1) as u16 + offset,
    };
    let frame = ((frames / 2) % 4) as u16; // animation has four stages : closed, semi, open, semi
    TILESET_SPRITES.draw_tile(
        p,
        Point {
            x: if frame == 0 { 2 } else { frame % 2 },
            y: if frame == 0 {
                0
            } else {
                match dir {
                    Direction::Up => 2,
                    Direction::Down => 3,
                    Direction::Right => 0,
                    Direction::Left => 1,
                }
            },
        },
        1,
        true,
    );
}

/// Clear ghost or player by redrawing the maze tiles around it.
pub fn clear_moveable(pos: Point, _steps: u8, _dir: &Direction, grid: &Grid, _is_ghost_home: bool) {
    ensure_maze_tile_cache();
    for dy in -1..=1 {
        for dx in -1..=1 {
            let tile_pos = Point {
                x: (pos.x as i16 + dx as i16).clamp(0, GRID_WIDTH as i16 - 1) as u16,
                y: (pos.y as i16 + dy as i16).clamp(0, GRID_HEIGHT as i16 - 1) as u16,
            };
            let tile_pixel_pos = abs_from_pos(tile_pos);
            match grid.get((tile_pos.x + tile_pos.y * GRID_WIDTH) as usize) {
                Some(Space::Point) => draw_space(tile_pixel_pos, Space::Point),
                Some(Space::Superball) => draw_space(tile_pixel_pos, Space::Superball),
                Some(Space::Fruit) => draw_space(tile_pixel_pos, Space::Fruit),
                Some(Space::Empty) | None => draw_space(tile_pixel_pos, Space::Empty),
                Some(Space::Wall) => {
                    let wall_tile_pos =
                        unsafe { MAZE_TILE_CACHE[(tile_pos.x + tile_pos.y * GRID_WIDTH) as usize] };
                    TILESET_WALLS.draw_tile(tile_pixel_pos, wall_tile_pos, 1, false);
                }
            }
        }
    }
}

/// Draws a ghost.
pub fn draw_ghost(
    pos: Point,
    steps: u8,
    dir: &Direction,
    frames: u32,
    wrapping: bool,
    gtype: &GhostType,
    house_state: HouseState,
    movement_mode: &MovementMode,
) {
    let mut np = abs_from_pos(pos);
    if house_state == HouseState::Inside {
        np.y = np.y + TILE_SIZE / 2;
    }

    let offset = match dir {
        Direction::Up | Direction::Down => 0,
        Direction::Right | Direction::Left => 1,
    };
    let p = Point {
        x: (np.x as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().0) as u16,
        y: (np.y as i16 - TILE_SIZE as i16 / 2 + steps as i16 * dir.to_vector().1) as u16 + offset,
    };
    match movement_mode {
        MovementMode::Frightened => TILESET_SPRITES.draw_tile(
            p,
            Point {
                x: 2 + ((frames / 4) % 2) as u16,
                y: 2,
            },
            1,
            true,
        ),
        MovementMode::FrightenedBlinking => TILESET_SPRITES.draw_tile(
            p,
            Point {
                x: 2 + ((frames / 4) % 2) as u16 + ((frames / 16) % 2) as u16 * 2,
                y: 2,
            },
            1,
            true,
        ),
        MovementMode::Scatter | MovementMode::Chase => TILESET_SPRITES.draw_tile(
            p,
            Point {
                x: match dir {
                    Direction::Up => 4,
                    Direction::Down => 6,
                    Direction::Right => 0,
                    Direction::Left => 2,
                } + ((frames / 4) % 2) as u16,
                y: match gtype {
                    GhostType::Blinky => 4,
                    GhostType::Pinky => 5,
                    GhostType::Inky => 6,
                    GhostType::Clyde => 7,
                },
            },
            1,
            true,
        ),
        MovementMode::Eaten => TILESET_SPRITES.draw_tile(
            p,
            Point {
                x: match dir {
                    Direction::Up => 4,
                    Direction::Down => 5,
                    Direction::Right => 2,
                    Direction::Left => 3,
                },
                y: 3,
            },
            1,
            true,
        ),
    }
}

/// Draws a given grid space.
pub fn draw_space(pos: Point, space: Space) {
    let tile_pos = match space {
        Space::Point => get_tile_position('.').unwrap(),
        Space::Superball => get_tile_position('°').unwrap(),
        Space::Empty | Space::Wall | Space::Fruit => get_tile_position(' ').unwrap(),
    };
    TILESET_WALLS.draw_tile(pos, tile_pos, 1, false);
}

pub fn draw_fruit(grid_pos: Point, fruit_type: u8) {
    let tile_pos = Point {
        x: fruit_type as u16,
        y: 8,
    };
    let mut abs_pos = abs_from_pos(grid_pos);
    abs_pos.y = abs_pos.y - TILE_SIZE / 2;
    TILESET_SPRITES.draw_tile(abs_pos, tile_pos, 1, true);
}

pub fn clear_potential_wrapping_stuff() {
    push_rect_uniform(
        Rect {
            x: X_GRID_OFFSET - TILE_SIZE * 2,
            y: 11 * TILE_SIZE,
            width: TILE_SIZE * 2,
            height: TILE_SIZE * 4,
        },
        Color::BLACK,
    );
    push_rect_uniform(
        Rect {
            x: X_GRID_OFFSET + GRID_WIDTH * TILE_SIZE,
            y: 11 * TILE_SIZE,
            width: TILE_SIZE * 2,
            height: TILE_SIZE * 4,
        },
        Color::BLACK,
    );
}

/// Death animation. Needs the killer ghost too.
pub fn draw_dead_pac(pac: &Player, ghost: &Ghost, grid: &Grid) {
    let np = abs_from_pos(pac.moveable.grid_position);
    let offset = match pac.moveable.direction {
        Direction::Up | Direction::Down => 0,
        Direction::Right | Direction::Left => 1,
    };
    let p = Point {
        x: (np.x as i16 - TILE_SIZE as i16 / 2
            + pac.moveable.steps as i16 * pac.moveable.direction.to_vector().0) as u16,
        y: (np.y as i16 - TILE_SIZE as i16 / 2
            + pac.moveable.steps as i16 * pac.moveable.direction.to_vector().1) as u16
            + offset,
    };

    msleep(750);
    for y in 0..2 {
        for x in 2..8 {
            clear_moveable(
                pac.moveable.grid_position,
                pac.moveable.steps as u8,
                &pac.moveable.direction,
                grid,
                false,
            );
            draw_ghost(
                ghost.moveable.grid_position,
                ghost.moveable.steps as u8,
                &ghost.moveable.direction,
                0,
                ghost.moveable.wrapping,
                &ghost.gtype,
                ghost.house_state,
                &ghost.movement_mode,
            );
            TILESET_SPRITES.draw_tile(p, Point { x, y }, 1, true);
            wait_for_vblank();
            wait_for_vblank();
            wait_for_vblank();
        }
    }
    clear_moveable(
        pac.moveable.grid_position,
        pac.moveable.steps as u8,
        &pac.moveable.direction,
        grid,
        false,
    );
    draw_ghost(
        ghost.moveable.grid_position,
        ghost.moveable.steps as u8,
        &ghost.moveable.direction,
        0,
        ghost.moveable.wrapping,
        &ghost.gtype,
        ghost.house_state,
        &ghost.movement_mode,
    );
}

const UI_Y_OFFSET: u16 = 5;
const UI_X_OFFSET: u16 = 2;
pub fn draw_constant_ui(high_score: u32) {
    draw_string(
        "LEVEL\0",
        Point {
            x: UI_X_OFFSET,
            y: UI_Y_OFFSET,
        },
        false,
        Color::WHITE,
        Color::BLACK,
    );
    draw_string(
        "SCORE\0",
        Point {
            x: UI_X_OFFSET,
            y: UI_Y_OFFSET + (SMALL_CHAR_HEIGHT + 2) * 2,
        },
        false,
        Color::WHITE,
        Color::BLACK,
    );
    draw_string(
        "HIGH\0",
        Point {
            x: SCREEN_WIDTH - UI_X_OFFSET - get_string_pixel_size("HIGH\0", false),
            y: UI_Y_OFFSET,
        },
        false,
        Color::WHITE,
        Color::BLACK,
    );
    let high_score_string = string_from_u32(high_score);
    draw_string(
        &high_score_string,
        Point {
            x: SCREEN_WIDTH - UI_X_OFFSET - get_string_pixel_size(&high_score_string, false),
            y: UI_Y_OFFSET + SMALL_CHAR_HEIGHT + 2,
        },
        false,
        Color::WHITE,
        Color::BLACK,
    );

    draw_string(
        "LIVES",
        Point {
            x: UI_X_OFFSET,
            y: SCREEN_HEIGHT - UI_Y_OFFSET - TILE_SIZE * 2 - SMALL_CHAR_HEIGHT - 2,
        },
        false,
        Color::WHITE,
        Color::BLACK,
    );
}

pub fn draw_score(score: u32) {
    draw_string(
        &string_from_u32(score),
        Point {
            x: UI_X_OFFSET,
            y: UI_X_OFFSET + (SMALL_CHAR_HEIGHT + 2) * 3,
        },
        false,
        Color::WHITE,
        Color::BLACK,
    );
}

pub fn draw_level(level: u16) {
    draw_string(
        &string_from_u16(level),
        Point {
            x: UI_X_OFFSET,
            y: UI_Y_OFFSET + (SMALL_CHAR_HEIGHT + 2),
        },
        false,
        Color::WHITE,
        Color::BLACK,
    );
}

pub fn draw_lives(lives: u8) {
    let mut pos = Point {
        x: 0,
        y: SCREEN_HEIGHT - UI_Y_OFFSET - TILE_SIZE * 2,
    };
    push_rect_uniform(
        Rect {
            x: pos.x,
            y: pos.y,
            width: TILE_SIZE * 2 * 3,
            height: TILE_SIZE * 2,
        },
        Color::BLACK,
    );
    for _ in 0..lives {
        TILESET_SPRITES.draw_tile(pos, Point { x: 1, y: 0 }, 1, true);
        pos.x += TILE_SIZE * 2;
    }
}

/// Draws the maze based on the current grid state.
/// Wall graphics are pulled from MAZE_TILE_CACHE, while items/empty spaces reflect the active Grid.
pub fn draw_maze(grid: &Grid) {
    ensure_maze_tile_cache();
    for line in 0..GRID_HEIGHT as usize {
        wait_for_vblank();
        for col in 0..GRID_WIDTH as usize {
            let idx = line * GRID_WIDTH as usize + col;
            let pixel_pos = abs_from_pos(Point {
                x: col as u16,
                y: line as u16,
            });

            match grid[idx] {
                Space::Wall => {
                    let tile_pos = unsafe { MAZE_TILE_CACHE[idx] };
                    TILESET_WALLS.draw_tile(pixel_pos, tile_pos, 1, false);
                }
                space => draw_space(pixel_pos, space),
            }
        }
    }
}

/// For the maze, determines the tile position based on the character written in the .txt file.
const fn get_tile_position(c: char) -> Option<Point> {
    match c {
        '.' => Some(Point { x: 13, y: 2 }),
        '°' => Some(Point { x: 15, y: 2 }),
        ' ' => Some(Point { x: 12, y: 2 }),
        '0'..='9' => Some(Point {
            x: (c as u8 - b'0') as u16,
            y: 0,
        }),
        'a'..='f' => Some(Point {
            x: (c as u8 - b'a') as u16 + 10,
            y: 0,
        }),
        'g'..='v' => Some(Point {
            x: (c as u8 - b'g') as u16,
            y: 1,
        }),
        'w'..='z' => Some(Point {
            x: (c as u8 - b'w') as u16,
            y: 2,
        }),
        'A'..='L' => Some(Point {
            x: (c as u8 - b'A') as u16 + 4,
            y: 2,
        }),
        _ => None, // Handle unexpected characters
    }
}
