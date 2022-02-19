use image::Rgb;

use crate::heat::Heat;
use crate::types::temp;

#[derive(Clone, Copy, PartialEq)]
pub struct Tile {
    mat_type: Heat,
    heat_energy: temp,
}

// constructors and stuff
impl Tile {
    pub const fn new(mat: Heat, energy: temp) -> Self {
        Self {
            mat_type: mat,
            heat_energy: energy,
        }
    }

    pub const fn new_sink(absorbtion_rate: temp, energy: temp) -> Self {
        Self {
            mat_type: Heat::Sink {
                absorbed_per_tick: absorbtion_rate,
            },
            heat_energy: energy,
        }
    }

    pub const fn new_source(production_rate: temp, energy: temp) -> Self {
        Self {
            mat_type: Heat::Source {
                produced_per_tick: production_rate,
            },
            heat_energy: energy,
        }
    }

    pub const fn new_conductor(energy: temp, transfer_rate: temp) -> Self {
        Self {
            mat_type: Heat::Conductor { rate: transfer_rate },
            heat_energy: energy,
        }
    }

    pub const fn const_default() -> Self {
        Self::new(Heat::Conductor { rate: 0.0 }, 0.0)
    }

}

impl Tile {
    pub fn view(&self, max_heat: temp) -> Rgb<u8> {
        let heat_color_coef = 255f32 / max_heat;
        //loss of precision is intentional
        let color = heat_color_coef * self.heat_energy.abs();

        if self.heat_energy == 0.0 {
            Rgb::<_>([0; 3])
        } else if self.heat_energy > 0.0 {
            Rgb::<_>([color as u8, 0, 0])
        } else {
            Rgb::<_>([0, 0, color as u8])
        }
    }

    pub fn get_type(&self) -> Heat {
        self.mat_type
    }

    pub fn set_heat(&mut self, heat: temp) {
        self.heat_energy = heat;
    }

    pub fn get_heat(&self) -> temp {
        self.heat_energy
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::const_default()
    }
}