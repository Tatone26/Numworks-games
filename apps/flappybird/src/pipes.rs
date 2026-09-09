use num_engine::spawn_sprite;
use numworks_utils::utils::randint;

use crate::{
    flappy_ui::{ANIM_PIPE_LIP_BOT, ANIM_PIPE_LIP_TOP, ANIM_PIPE_SHAFT, TILESET_TILE_SIZE},
    game::{GameSprite, SCREEN_ROWS, TILE_SIZE, VIEW_SCREEN_W, VIEW_SCREEN_X},
};

const SHAFT_HEIGHT_PX: i16 = (SCREEN_ROWS * TILE_SIZE) as i16; // 220 px

pub struct PipePair<'a> {
    pub gap_y: i16,
    pub gap_size: i16,
    pub active: bool,
    pub scored: bool,
    pub spr_top_shaft: GameSprite<'a>,
    pub spr_top_lip: GameSprite<'a>,
    pub spr_bot_lip: GameSprite<'a>,
    pub spr_bot_shaft: GameSprite<'a>,
}

impl<'a> PipePair<'a> {
    pub fn new(gap_size: i16, speed: f32) -> Self {
        let spawn_x = (VIEW_SCREEN_X + VIEW_SCREEN_W) as i16;
        let gap_y = Self::random_gap(gap_size);
        let collar_x = spawn_x - TILESET_TILE_SIZE as i16;

        let mut spr_top_shaft = spawn_sprite!(
            &ANIM_PIPE_SHAFT,
            [spawn_x, (gap_y - 20) - SHAFT_HEIGHT_PX],
            10
        );
        let mut spr_top_lip = spawn_sprite!(&ANIM_PIPE_LIP_TOP, [collar_x, gap_y - 20], 11);
        let mut spr_bot_lip = spawn_sprite!(&ANIM_PIPE_LIP_BOT, [collar_x, gap_y + gap_size], 11);
        let mut spr_bot_shaft =
            spawn_sprite!(&ANIM_PIPE_SHAFT, [spawn_x, gap_y + gap_size + 20], 10);

        let s = [-speed, 0.0];
        spr_top_shaft.set_speed(s);
        spr_top_lip.set_speed(s);
        spr_bot_lip.set_speed(s);
        spr_bot_shaft.set_speed(s);

        Self {
            gap_y,
            gap_size,
            active: false,
            scored: false,
            spr_top_shaft,
            spr_top_lip,
            spr_bot_lip,
            spr_bot_shaft,
        }
    }

    fn random_gap(gap_size: i16) -> i16 {
        // Gap begins between y=20 and y=180 - gap_size in world space
        randint(20, (200 - 20 - gap_size) as u32) as i16
    }

    pub fn warp_to(&mut self, target_x: i16) {
        self.gap_y = Self::random_gap(self.gap_size);
        self.scored = false;

        let collar_x = target_x - TILESET_TILE_SIZE as i16;

        self.spr_top_shaft.set_position(
            [target_x, (self.gap_y - 20) - SHAFT_HEIGHT_PX],
            Some([0.0, 0.0]),
        );
        self.spr_top_lip
            .set_position([collar_x, self.gap_y - 20], Some([0.0, 0.0]));
        self.spr_bot_lip
            .set_position([collar_x, self.gap_y + self.gap_size], Some([0.0, 0.0]));
        self.spr_bot_shaft.set_position(
            [target_x, self.gap_y + self.gap_size + 20],
            Some([0.0, 0.0]),
        );
    }

    pub fn set_speed(&mut self, speed: f32) {
        let s = [-speed, 0.0];
        self.spr_top_shaft.set_speed(s);
        self.spr_top_lip.set_speed(s);
        self.spr_bot_lip.set_speed(s);
        self.spr_bot_shaft.set_speed(s);
    }

    pub fn update(&mut self, frame: u32) {
        self.spr_top_shaft.update(frame);
        self.spr_top_lip.update(frame);
        self.spr_bot_lip.update(frame);
        self.spr_bot_shaft.update(frame);
    }

    pub fn commit_frame(&mut self) {
        self.spr_top_shaft.commit_frame();
        self.spr_top_lip.commit_frame();
        self.spr_bot_lip.commit_frame();
        self.spr_bot_shaft.commit_frame();
    }

    #[inline(always)]
    pub fn x(&self) -> i16 {
        self.spr_top_shaft.position[0]
    }
}
