//! Declarative macros for compile-time asset definition and layout generation.

/// Declares a static [`Animation`](crate::texture::Animation) in Flash (ROM).
///
/// Stores frame slices and texture metadata directly in `.rodata`, using **zero bytes of RAM**.
///
/// # Syntax
/// ```rust
/// define_anim!(
///     [pub] NAME,
///     tileset_ref,
///     width_in_tiles,
///     height_in_tiles,
///     transparency_bool,
///     speed_ticks,
///     [ (tx0, ty0), (tx1, ty1), ... ]
/// );
///
/// Automatically sets `sheet_w = width` and `sheet_h = height` for standard non-repeating sprites.
///
#[macro_export]
macro_rules! define_anim {
    (
        $vis:vis $name:ident,
        $tileset:expr,
        $w:expr,
        $h:expr,
        $trans:expr,
        $speed:expr,
        [ $( ($tx:expr, $ty:expr) ),* $(,)? ]
    ) => {
        $crate::define_repeating_anim!(
            $vis $name,
            $tileset,
            $w,
            $h,
            $w,
            $h,
            $trans,
            $speed,
            [ $( ($tx, $ty) ),* ]
        );
    };
}

/// Declares a static Animation that repeats a (sheet_w x sheet_h) tile pattern across (width x height) tiles.
#[macro_export]
macro_rules! define_repeating_anim {
    (
        $vis:vis $name:ident,
        $tileset:expr,
        $w:expr,
        $h:expr,
        $sheet_w:expr,
        $sheet_h:expr,
        $trans:expr,
        $speed:expr,
        [ $( ($tx:expr, $ty:expr) ),* $(,)? ]
    ) => {
        $vis static $name: $crate::texture::Animation<'static> = $crate::texture::Animation {
            desc: $crate::texture::TextureDescriptor {
                tileset: $tileset,
                width: $w,
                height: $h,
                sheet_w: $sheet_w,
                sheet_h: $sheet_h,
                scaling: 1,
                transparency: $trans,
            },
            frames: &[
                $( $crate::texture::FrameCoord { tx: $tx, ty: $ty }, )*
            ],
            speed: $speed,
        };
    };
}

/// Spawns a [`Sprite`](crate::sprite::Sprite) instance from an animation reference.
///
/// # Syntax
/// ```rust
/// let player = spawn_sprite!(&PLAYER_ANIM, [x, y], z_index);
/// ```
#[macro_export]
macro_rules! spawn_sprite {
    ($anim:expr, $pos:expr, $z:expr $(,)?) => {
        $crate::sprite::Sprite::new($anim, $pos, $z)
    };
}

/// Populates a mutable slice buffer using a visual ASCII art string.
///
/// Trailing newlines advance the row; spaces and carriage returns are ignored.
///
/// # Syntax
/// ```rust
/// ascii_tilemap!(
///     tileset: &TILESET,
///     buffer: &mut map_data,
///     cols: 32,
///     rows: 24,
///     z: 10,
///     transparent: true,
///     parallax: Parallax::FOREGROUND,
///     wrap: false,
///     mapping: {
///         b'.' => None,
///         b'G' => Some([0, 4]),
///         b'P' => Some([1, 1]),
///     },
///     layout: "\
/// ....
/// GGGG"
/// );
/// ```
#[macro_export]
macro_rules! ascii_tilemap {
    (
        tileset: $tileset:expr,
        buffer: $buf:expr,
        cols: $cols:expr,
        rows: $rows:expr,
        z: $z:expr,
        transparent: $trans:expr,
        parallax: $parallax:expr,
        wrap: $wrap:expr,
        mapping: { $( $char:pat => $tile:expr ),* $(,)? },
        layout: $layout:expr $(,)?
    ) => {{
        let mut cur_c: usize = 0;
        let mut cur_r: usize = 0;

        for b in $layout.bytes() {
            match b {
                b'\n' => {
                    cur_r += 1;
                    cur_c = 0;
                }
                b'\r' | b' ' | b'\t' => continue,
                ch => {
                    if cur_c < $cols && cur_r < $rows {
                        let tile_coord: Option<[u8; 2]> = match ch {
                            $( $char => $tile, )*
                            _ => None,
                        };
                        $buf[cur_r * $cols + cur_c] = tile_coord;
                        cur_c += 1;
                    }
                }
            }
        }

        $crate::tilemap::Tilemap::new(
            $tileset, $buf, $cols, $rows, &[], $z, $trans, $parallax, $wrap,
        )
    }};
}

/// Fills rectangular bounds or horizontal spans of a Tilemap with a single tile.
///
/// # Syntax
/// ```rust
/// // Horizontal ground span:
/// tile_span!(map, row: 23, cols: 0..32, tile: Some([0, 4]));
///
/// // Rectangular block:
/// tile_rect!(map, cols: 5..7, rows: 3..5, tile: Some([1, 0]));
/// ```
#[macro_export]
macro_rules! tile_span {
    ($map:expr, row: $row:expr, cols: $cols:expr, tile: $tile:expr $(,)?) => {{
        for col in $cols {
            $map.set_tile(col, $row, $tile);
        }
    }};
}

#[macro_export]
macro_rules! tile_rect {
    ($map:expr, cols: $cols:expr, rows: $rows:expr, tile: $tile:expr $(,)?) => {{
        for r in $rows {
            for c in $cols.clone() {
                $map.set_tile(c, r, $tile);
            }
        }
    }};
}
