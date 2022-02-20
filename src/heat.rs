use crate::types::temp;

#[derive(Clone, Copy, PartialEq)]
pub enum Heat {
    /// heat source. tile temp is used as max to set surrounding blocks to and should not be changed after initialization
    Source {
        produced_per_tick: temp,
    },
    Sink {
        absorbed_per_tick: temp,
    },
    /// rate is the percentage of the temperature difference to cover every tick out of 100
    Conductor {
        rate: temp,
    },
}
