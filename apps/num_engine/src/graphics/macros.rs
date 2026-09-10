//! Declarative macros for compile-time graphics, animations, sprites, and tile buffers.

pub trait IntoWrapMode {
    fn into_wrap_mode(self) -> crate::graphics::tilemap::WrapMode;
}

impl IntoWrapMode for crate::graphics::tilemap::WrapMode {
    #[inline(always)]
    fn into_wrap_mode(self) -> crate::graphics::tilemap::WrapMode {
        self
    }
}

impl IntoWrapMode for bool {
    #[inline(always)]
    fn into_wrap_mode(self) -> crate::graphics::tilemap::WrapMode {
        if self {
            crate::graphics::tilemap::WrapMode::Both
        } else {
            crate::graphics::tilemap::WrapMode::None
        }
    }
}

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
        $vis static $name: $crate::graphics::texture::Animation<'static> = $crate::graphics::texture::Animation {
            desc: $crate::graphics::texture::TextureDescriptor {
                tileset: $tileset,
                width: $w,
                height: $h,
                sheet_w: $sheet_w,
                sheet_h: $sheet_h,
                scaling: 1,
                transparency: $trans,
            },
            frames: &[
                $( $crate::graphics::texture::FrameCoord { tx: $tx, ty: $ty }, )*
            ],
            speed: $speed,
        };
    };
}

/// Constructs a Sprite with absolute coordinates and optional relative offset.
#[macro_export]
macro_rules! spawn_sprite {
    ($anim:expr, $pos:expr, $z:expr $(,)?) => {
        $crate::graphics::Sprite::new($anim, $pos, $z)
    };
    ($anim:expr, $pos:expr, $z:expr, offset: [$ox:expr, $oy:expr] $(,)?) => {
        $crate::graphics::Sprite::new($anim, $pos, $z).with_offset([$ox as i16, $oy as i16])
    };
}

/// Allocates an off-stack static tile buffer for a tilemap layer.
#[macro_export]
macro_rules! define_tilemap_buffer {
    ($vis:vis $name:ident, $cols:expr, $rows:expr) => {
        $vis static mut $name: [Option<[u8; 2]>; ($cols) * ($rows)] = [None; ($cols) * ($rows)];
    };
}

/// Fills a horizontal span of tiles in a Tilemap.
#[macro_export]
macro_rules! tile_span {
    ($map:expr, row: $row:expr, cols: $cols:expr, tile: $tile:expr $(,)?) => {{
        for col in $cols {
            $map.set_tile(col, $row, $tile);
        }
    }};
}

/// Fills a rectangular block of tiles in a Tilemap.
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

/// Parses ASCII art layouts into an independent Tilemap.
#[macro_export]
macro_rules! ascii_tilemap {
    (
        tileset: $tileset:expr,
        buffer: $buf:expr,
        cols: $cols:expr,
        rows: $rows:expr,
        $(offset: [$ox:expr, $oy:expr],)?
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

        let mut map = $crate::graphics::Tilemap::new(
            $tileset,
            $buf,
            $cols,
            $rows,
            &[],
            $z,
            $trans,
            $parallax,
            $crate::graphics::macros::IntoWrapMode::into_wrap_mode($wrap),
        );
        $(
            map.set_offset([$ox as i16, $oy as i16]);
        )?
        map
    }};
}
