use crate::types::temp;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Heat {
    /// heat source. tile temp is used as max to set surrounding blocks to and should not be changed after initialization
    Source { produced_per_tick: temp },
    Sink { absorbed_per_tick: temp },
    Conductor { rate: temp },
}
