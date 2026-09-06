use numworks_utils::eadk::Point;

use crate::world::Tile;

const RIVER_TILE: Tile = Tile {
    grid_point: Point { x: 0, y: 0 },
    is_killer: true,
    has_obstacle: false,
    tileset_index: Point { x: 0, y: 1 },
};

const ROAD_TILE: Tile = Tile {
    grid_point: Point { x: 0, y: 0 },
    is_killer: false,
    has_obstacle: false,
    tileset_index: Point { x: 0, y: 3 },
};

const GRASS_TILE: Tile = Tile {
    grid_point: Point { x: 0, y: 0 },
    is_killer: false,
    has_obstacle: false,
    tileset_index: Point { x: 0, y: 2 },
};

const TREE_GRASS_TILE: Tile = Tile {
    grid_point: Point { x: 0, y: 0 },
    is_killer: false,
    has_obstacle: true,
    tileset_index: Point { x: 1, y: 2 },
};
