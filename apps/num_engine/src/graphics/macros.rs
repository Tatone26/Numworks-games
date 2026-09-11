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

/// Constructs an [`Emitter`](crate::graphics::particles::Emitter) with zone-based randomized seeding.
#[macro_export]
macro_rules! emitter {
    // 1. Explicit independent dimensions [width, height]
    (
        area: [$x:expr, $y:expr, $w:expr, $h:expr],
        rate: $rate:expr,
        interval: $interval:expr,
        vx: [$min_vx:expr, $max_vx:expr],
        vy: [$min_vy:expr, $max_vy:expr],
        width: [$min_w:expr, $max_w:expr],
        height: [$min_h:expr, $max_h:expr],
        color: $color:expr,
        life: [$min_life:expr, $max_life:expr] $(,)?
    ) => {{
        let mut em = $crate::graphics::particles::Emitter::empty();
        em.area = [$x as i16, $y as i16, $w as i16, $h as i16];
        em.rate = $rate as u8;
        em.interval = $interval as u8;
        em.vx_range = [$min_vx as f32, $max_vx as f32];
        em.vy_range = [$min_vy as f32, $max_vy as f32];
        em.size_x_range = [$min_w as u8, $max_w as u8];
        em.size_y_range = [$min_h as u8, $max_h as u8];
        em.color = $color;
        em.life_range = [$min_life as u8, $max_life as u8];
        em.enabled = true;
        em
    }};

    // 2. Streaks / Squares (Single size metric, thickness defaults to 1 px)
    (
        area: [$x:expr, $y:expr, $w:expr, $h:expr],
        rate: $rate:expr,
        interval: $interval:expr,
        vx: [$min_vx:expr, $max_vx:expr],
        vy: [$min_vy:expr, $max_vy:expr],
        size: [$min_size:expr, $max_size:expr],
        color: $color:expr,
        life: [$min_life:expr, $max_life:expr] $(,)?
    ) => {{
        $crate::emitter!(
            area: [$x, $y, $w, $h],
            rate: $rate,
            interval: $interval,
            vx: [$min_vx, $max_vx],
            vy: [$min_vy, $max_vy],
            width: [$min_size, $max_size],
            height: [1, 1],
            color: $color,
            life: [$min_life, $max_life],
        )
    }};

    // 3. Constant scalar size
    (
        area: [$x:expr, $y:expr, $w:expr, $h:expr],
        rate: $rate:expr,
        interval: $interval:expr,
        vx: [$min_vx:expr, $max_vx:expr],
        vy: [$min_vy:expr, $max_vy:expr],
        size: $size:expr,
        color: $color:expr,
        life: $life:expr $(,)?
    ) => {{
        let s = $size as u8;
        let l = $life as u8;
        $crate::emitter!(
            area: [$x, $y, $w, $h],
            rate: $rate,
            interval: $interval,
            vx: [$min_vx, $max_vx],
            vy: [$min_vy, $max_vy],
            width: [s, s],
            height: [1, 1],
            color: $color,
            life: [l, l],
        )
    }};
}
