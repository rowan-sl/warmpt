pub mod heat;
pub mod tile;
pub mod types;
pub mod world;

extern crate image;
extern crate pretty_env_logger;
extern crate stati;
extern crate yaml_rust;
#[macro_use]
extern crate log;

use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};

use image::codecs::gif::GifEncoder;
use image::{codecs::gif::Repeat, Delay, DynamicImage, Frame};
use yaml_rust::{Yaml, YamlLoader};
use stati::{BarManager, bars, prelude::*};

use tile::Tile;
use world::{World, WorldBuilder};

fn parse_tile(src: &Vec<Yaml>) -> Option<Tile> {
    if src.len() != 3 {
        None?
    }
    parse_tile_from_parts(&src[0], &src[1], &src[2])
}

fn parse_tile_from_parts(p1: &Yaml, p2: &Yaml, p3: &Yaml) -> Option<Tile> {
    let arg1 = p2.as_f64()? as f32;
    let arg2 = p3.as_f64()? as f32;
    Some(match p1.as_str()? {
        "conductor" => Tile::new_conductor(arg1, arg2),
        "source" => Tile::new_source(arg1, arg2),
        "sink" => Tile::new_sink(arg1, arg2),
        _ => None?, /* consise return :0 */
    })
}

#[derive(Clone, Debug)]
struct Config {
    out_file: String,
    gif_repeat: bool,
    sim_steps: u64,
    sim_substeps: u64,
    max_display_heat: f32,
    frame_delay: Delay,
}

fn load_world(config: Yaml) -> Option<(World, Config)> {
    let out_file = config["render_file"].as_str()?;
    let gif_repeat = config["render_repeat"].as_bool()?;
    let ms_per_frame = config["ms_per_frame"].as_i64()? as u64;
    let frame_delay = Delay::from_saturating_duration(Duration::from_millis(ms_per_frame));
    let sim_steps = config["sim_steps"].as_i64()? as u64;
    let sim_substeps = config["sim_substeps"].as_i64()? as u64;
    let max_display_heat = config["max_display_heat"].as_f64()? as f32;
    let world_size_raw = config["world_size"].as_vec()?;
    info!("Boaded world properties");
    let world_size = (
        world_size_raw[0].as_i64()? as u64,
        world_size_raw[1].as_i64()? as u64,
    );
    let default_tile = parse_tile(config["default_tile"].as_vec()?)?;

    let mut world_builder =
        WorldBuilder::with_default_tile(world_size.0 as usize, world_size.1 as usize, default_tile);

    let build_instructions_raw = config["build_instructions"].as_vec()?;

    info!("Building world..");

    for raw_inst in build_instructions_raw {
        trace!("Executing build instruction: {:?}", raw_inst);
        let inst = raw_inst.as_vec()?;
        if inst.len() == 0 {
            panic!()
        }
        match inst[0].as_str()? {
            "set" => {
                if inst.len() != 6 {
                    panic!()
                }
                let x = inst[1].as_i64()? as usize;
                let y = inst[2].as_i64()? as usize;
                let tile = parse_tile_from_parts(&inst[3], &inst[4], &inst[5])?;
                world_builder.set(x, y, tile);
            }
            "set_sect_x" => {
                if inst.len() != 7 {
                    panic!()
                }
                let x_s = inst[1].as_i64()? as usize;
                let x_e = inst[2].as_i64()? as usize;
                let y = inst[3].as_i64()? as usize;
                let tile = parse_tile_from_parts(&inst[4], &inst[5], &inst[6])?;
                world_builder.set_sect_x(x_s, x_e, y, tile);
            }
            "set_sect_y" => {
                if inst.len() != 7 {
                    panic!()
                }
                let y_s = inst[1].as_i64()? as usize;
                let y_e = inst[2].as_i64()? as usize;
                let x = inst[3].as_i64()? as usize;
                let tile = parse_tile_from_parts(&inst[4], &inst[5], &inst[6])?;
                world_builder.set_sect_y(y_s, y_e, x, tile);
            }
            _ => panic!(),
        }
    }

    Some((
        world_builder.build(),
        Config {
            out_file: out_file.into(),
            gif_repeat,
            sim_steps,
            sim_substeps,
            max_display_heat,
            frame_delay,
        },
    ))
}

fn load_config<P: AsRef<Path>>(path: P) -> anyhow::Result<Yaml> {
    let mut conf_file = OpenOptions::new().read(true).create(false).open(path)?;
    let mut conf_txt = String::new();
    conf_file.read_to_string(&mut conf_txt)?;
    Ok(YamlLoader::load_from_str(&conf_txt)?.remove(0))
}

fn get_path() -> anyhow::Result<String> {
    let args = std::env::args().collect::<Vec<String>>();
    let path = if let Some(v) = args.get(1) {
        v
    } else {
        anyhow::bail!("Missing config argument!");
    };
    Ok(path.clone())
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::try_init_custom_env("WARMPT_LOG")?;
    info!("Initializing simulation");
    let path = get_path()?;
    debug!("Loading configuration from {path}..");
    let conf_file = load_config(path)?;
    debug!("Loaded configuration");
    debug!("Loading world...");
    let (mut world, config) = if let Some(v) = load_world(conf_file) {
        v
    } else {
        error!("Invalid Configuration!");
        anyhow::bail!("Could not load world! (invalid config)")
    };
    debug!("Loaded world");

    info!("Creating render file {}", config.out_file);
    let render_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(config.out_file)?;
    let mut encoder = GifEncoder::new(render_file);
    encoder
        .set_repeat(if config.gif_repeat {
            Repeat::Infinite
        } else {
            Repeat::Finite(0)
        })?;

    debug!("Created render file");

    info!("Starting simulation...");

    debug!("Simulation running for {} frames, with {} steps per frame", config.sim_steps, config.sim_substeps);

    let before = Instant::now();

    let mut bman = BarManager::new();
    for _ in (0..config.sim_steps).display_bar(bman.register(bars::SimpleBar::new(&"Simulating", 0))) {
        for _ in 0..config.sim_substeps {
            world.tick();
        }
        bman.print();
        let img = DynamicImage::ImageRgb8(world.observe(config.max_display_heat)).into_rgba8();
        encoder
            .encode_frame(Frame::from_parts(img, 0, 0, config.frame_delay))
            .unwrap();
    }

    let after = Instant::now();

    debug!("Simulation ran in {:?}", after - before);

    info!("done!");

    Ok(())
}
