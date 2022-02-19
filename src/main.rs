pub mod heat;
pub mod tile;
pub mod world;
pub mod types;

extern crate image;
extern crate stati;

use std::fs::OpenOptions;
use std::time::Duration;

use image::codecs::gif::GifEncoder;
use image::{Frame, Delay, DynamicImage};

use tile::Tile;
use types::temp;
use world::WorldBuilder;

const FRAMETIME: u64 = 200;
const SIM_STEPS: u32 = 100;
const MAX_HEAT: temp = 1000;

fn main() {
    let mut world = WorldBuilder::<20, 20>::with_default_tile(Tile::new_conductor(0, 10))
        .set(0, 0, Tile::new_source(40, 700))
        // .set(19, 19, Tile::new_sink(1, 0))
        .build();

    let file = OpenOptions::new().write(true).create(true).open("out.gif").unwrap();
    let mut genc = GifEncoder::new(file);

    for _ in 0..SIM_STEPS {
        world.tick();
        let img = DynamicImage::ImageRgb8(world.observe(MAX_HEAT)).into_rgba8();
        genc.encode_frame(Frame::from_parts(img, 0, 0, Delay::from_saturating_duration(Duration::from_millis(FRAMETIME)))).unwrap();
    }
}
