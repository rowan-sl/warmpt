use image::RgbImage;

use crate::{tile::Tile, types::temp, heat::Heat};

pub fn surounding(x: usize, y: usize, mx: usize, my: usize) -> Vec<(usize, usize)> {
    let ix = x as isize;
    let iy = y as isize;
    let pos_around = [
        (ix + 1, iy),
        (ix, iy + 1),
        (ix - 1, iy),
        (ix, iy - 1),
    ];
    let mut res = vec![];
    for v in pos_around {
        if v.0 >= 0 && v.0 < mx as isize && v.1 >= 0 && v.1 < my as isize {
            res.push((v.0 as usize, v.1 as usize))
        }
    }
    res
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct World<const X: usize, const Y: usize> {
    tiles: [[Tile; Y]; X],
}

impl<const X: usize, const Y: usize> World<X, Y> {
    /// display the [`World`] as a image representing the heat of tiles at a 1:1 tile:pixel ratio
    /// 
    /// ## Panics
    /// if X and Y cannot be converted to u32
    pub fn observe(&self, max_heat: temp) -> RgbImage {
        RgbImage::from_fn(u32::try_from(X).unwrap(), u32::try_from(Y).unwrap(), |x, y| {
            self.tiles[x as usize][y as usize].view(max_heat)
        })
    }

    pub fn tick(&mut self) {
        for x in 0..X {
            for y in 0..Y {
                let heat_at = self.tiles[x][y].get_heat();
                let type_at = self.tiles[x][y].get_type();
                match type_at {
                    Heat::Source {produced_per_tick} => {
                        for cord in surounding(x, y, X, Y) {
                            let h = self.tiles[cord.0][cord.1].get_heat();
                            if h < heat_at {
                                self.tiles[cord.0][cord.1].set_heat(h + produced_per_tick);
                            }
                        }
                    }
                    Heat::Sink {absorbed_per_tick} => {
                        for cord in surounding(x, y, X, Y) {
                            let h = self.tiles[cord.0][cord.1].get_heat();
                            if h > heat_at {
                                self.tiles[cord.0][cord.1].set_heat(h - absorbed_per_tick);
                            }
                        }
                    }
                    Heat::Conductor { rate } => {
                        let mut new_heat_at = heat_at;
                        let s = surounding(x, y, X, Y);
                        let s_l = s.len();
                        let transfer_per = rate / s_l as isize;
                        for cord in s {
                            let h = self.tiles[cord.0][cord.1].get_heat();
                            if new_heat_at > h && new_heat_at - transfer_per >= h {
                                let new_h = h + transfer_per;
                                self.tiles[cord.0][cord.1].set_heat(new_h);
                                new_heat_at -= transfer_per;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldBuilder<const X: usize, const Y: usize> {
    world: World<X, Y>,
}

impl<const X: usize, const Y: usize> WorldBuilder<X, Y> {
    pub const fn new() -> Self {
        Self {
            world: World {
                tiles: [[Tile::const_default(); Y]; X],
            },
        }
    }

    pub const fn with_default_tile(dt: Tile) -> Self {
        Self {
            world: World {
                tiles: [[dt; Y]; X],
            },
        }
    }

    pub const fn set(mut self, x: usize, y: usize, t: Tile) -> Self {
        self.world.tiles[x][y] = t;
        self
    }

    pub const fn get(&self, x: usize, y: usize) -> Tile {
        self.world.tiles[x][y]
    }

    /// ## Notes:
    /// this is NOT a const fn, so using this will cease the usefullness of this being a const constructor
    pub fn set_sect_x(mut self, x_s: usize, x_e: usize, y: usize, t: Tile) -> Self {
        for x in x_s..=x_e {
            self.world.tiles[x][y] = t;
        }
        self
    }

    /// ## Notes:
    /// this is NOT a const fn, so using this will cease the usefullness of this being a const constructor
    pub fn set_sect_y(mut self, y_s: usize, y_e: usize, x: usize, t: Tile) -> Self {
        for y in y_s..=y_e {
            self.world.tiles[x][y] = t;
        }
        self
    }

    pub const fn build(self) -> World<X, Y> {
        self.world
    }
}

impl<const X: usize, const Y: usize> From<WorldBuilder<X, Y>> for World<X, Y> {
    fn from(wb: WorldBuilder<X, Y>) -> Self {
        wb.build()
    }
}
