use image::RgbImage;

use crate::{heat::Heat, tile::Tile, types::temp};

pub fn surounding(x: usize, y: usize, mx: usize, my: usize) -> Vec<(usize, usize)> {
    let ix = x as isize;
    let iy = y as isize;
    let pos_around = [(ix + 1, iy), (ix, iy + 1), (ix - 1, iy), (ix, iy - 1)];
    let mut res = vec![];
    for v in pos_around {
        if v.0 >= 0 && v.0 < mx as isize && v.1 >= 0 && v.1 < my as isize {
            res.push((v.0 as usize, v.1 as usize))
        }
    }
    res
}

#[derive(Clone, PartialEq)]
pub struct World {
    tiles: Vec<Vec<Tile>>,
    x_len: usize,
    y_len: usize,
}

impl World {
    /// display the [`World`] as a image representing the heat of tiles at a 1:1 tile:pixel ratio
    ///
    /// ## Panics
    /// if X and Y cannot be converted to u32
    pub fn observe(&self, max_heat: temp) -> RgbImage {
        RgbImage::from_fn(
            u32::try_from(self.x_len).unwrap(),
            u32::try_from(self.y_len).unwrap(),
            |x, y| self.tiles[x as usize][y as usize].view(max_heat),
        )
    }

    pub fn tick(&mut self) {
        for x in 0..self.x_len {
            for y in 0..self.y_len {
                let heat_at = self.tiles[x][y].get_heat();
                let type_at = self.tiles[x][y].get_type();
                match type_at {
                    Heat::Source { produced_per_tick } => {
                        for cord in surounding(x, y, self.x_len, self.y_len) {
                            // get the heat
                            let h = self.tiles[cord.0][cord.1].get_heat();
                            // if the heat is less that the max produced, then increase that tiles heat by the produced ammount
                            if h < heat_at {
                                self.tiles[cord.0][cord.1].set_heat(h + produced_per_tick);
                            }
                        }
                    }
                    Heat::Sink { absorbed_per_tick } => {
                        for cord in surounding(x, y, self.x_len, self.y_len) {
                            // get the heat
                            let h = self.tiles[cord.0][cord.1].get_heat();
                            // if the heat is larger than the current heat, take heat away
                            if h > heat_at {
                                self.tiles[cord.0][cord.1].set_heat(h - absorbed_per_tick);
                            }
                        }
                    }
                    Heat::Conductor { rate } => {
                        // percent of heat diff to cover every tick
                        let rate_precent = rate / 100.0;
                        // new heat after all operatoins
                        let mut new_heat_at = heat_at;
                        // get the surrounding tiles
                        let surrounding_cords = surounding(x, y, self.x_len, self.y_len);
                        // number of surrounding tiles
                        let num_surrounding = surrounding_cords.len();
                        // for each nearby tile
                        for surrounding_tile in surrounding_cords {
                            //get the heat of that tile
                            let surrounding_tile_heat =
                                self.tiles[surrounding_tile.0][surrounding_tile.1].get_heat();
                            //get the difference in heat
                            let heat_dif = new_heat_at - surrounding_tile_heat;

                            if heat_dif > 0.0 {
                                // if the other tiles heat is larger
                                let mut change = heat_dif * rate_precent;
                                // account for the number of nearby tiles heat will go to
                                change /= num_surrounding as f32;

                                // increase the heat by change, changing behavior if it is a sink or source
                                match self.tiles[surrounding_tile.0][surrounding_tile.1].get_type()
                                {
                                    Heat::Conductor { .. } => {
                                        self.tiles[surrounding_tile.0][surrounding_tile.1]
                                            .set_heat(surrounding_tile_heat + change);
                                    }
                                    _ => {
                                        //do nothing, as this satisfys the behavior of bolth sources and sinks
                                    }
                                }
                                // decrease own heat
                                new_heat_at -= change;
                            }
                        }
                        // update the heat at the current tiles
                        self.tiles[x][y].set_heat(new_heat_at);
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct WorldBuilder {
    world: World,
}

impl WorldBuilder {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            world: World {
                tiles: std::iter::repeat(
                    std::iter::repeat(Tile::default())
                        .take(y)
                        .collect::<Vec<Tile>>(),
                )
                .take(x)
                .collect::<Vec<Vec<Tile>>>(),
                x_len: x,
                y_len: y,
            },
        }
    }

    pub fn with_default_tile(x: usize, y: usize, dt: Tile) -> Self {
        Self {
            world: World {
                tiles: std::iter::repeat(std::iter::repeat(dt).take(y).collect::<Vec<Tile>>())
                    .take(x)
                    .collect::<Vec<Vec<Tile>>>(),
                x_len: x,
                y_len: y,
            },
        }
    }

    pub fn set(&mut self, x: usize, y: usize, t: Tile) {
        self.world.tiles[x][y] = t;
    }

    pub fn get(&self, x: usize, y: usize) -> Tile {
        self.world.tiles[x][y]
    }

    pub fn set_sect_x(&mut self, x_s: usize, x_e: usize, y: usize, t: Tile) {
        for x in x_s..=x_e {
            self.world.tiles[x][y] = t;
        }
    }

    pub fn set_sect_y(&mut self, y_s: usize, y_e: usize, x: usize, t: Tile) {
        for y in y_s..=y_e {
            self.world.tiles[x][y] = t;
        }
    }

    pub fn build(self) -> World {
        self.world
    }
}

impl From<WorldBuilder> for World {
    fn from(wb: WorldBuilder) -> Self {
        wb.build()
    }
}
