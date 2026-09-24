//! Explicit mechanisms supplement the unchanged pilot rather than retuning it.

use super::{HuntConfig, HunterSeed};
use crate::{
    Cell, Daylight, FounderTrait, GrazerPlacement, GrazerSeed, GridBounds, MobileScenario, Motion,
    PatchPlacement, PatchSeed, PopulationConfig, RestPolicy, Scenario, SimId, SpatialConfig,
    UpkeepTrait,
};
impl MobileScenario {
    pub fn hunting_contention() -> Self {
        let cell = Cell { x: 0, y: 0 };
        let motion = Motion {
            sensing_radius: 1,
            max_cells_per_tick: 0,
            travel_units_per_cell: 1,
        };
        let scenario = Scenario {
            daylight: Daylight::new(1, 0).expect("dark day"),
            grazers: [1, 2]
                .into_iter()
                .map(|id| GrazerSeed {
                    id: SimId(id),
                    reserve_units: 5,
                    capacity_units: 8,
                    maintenance_units_per_tick: 1,
                    meal_units_per_tick: 2,
                    feeding_site: None,
                })
                .collect(),
            patches: vec![PatchSeed {
                id: SimId(1000),
                biomass_units: 10,
                capacity_units: 10,
                growth_units_per_lit_tick: 0,
            }],
            history_limit: 128,
        };
        let population = PopulationConfig::new(
            [1, 2]
                .into_iter()
                .map(|id| FounderTrait {
                    id: SimId(id),
                    upkeep: UpkeepTrait::new(1).expect("trait1"),
                })
                .collect(),
        );
        let space = SpatialConfig {
            bounds: GridBounds {
                width_cells: 2,
                height_cells: 1,
            },
            grazers: [1, 2]
                .into_iter()
                .map(|id| GrazerPlacement {
                    id: SimId(id),
                    cell,
                    motion,
                })
                .collect(),
            patches: vec![PatchPlacement {
                id: SimId(1000),
                cell,
            }],
            newborn_motion: motion,
        };
        Self::new(scenario, population, space)
            .with_rest(RestPolicy::default())
            .with_hunters(HuntConfig::new(
                [100, 101]
                    .into_iter()
                    .map(|id| HunterSeed {
                        id: SimId(id),
                        reserve_units: 4,
                        capacity_units: 8,
                        maintenance_units_per_tick: 1,
                        attack_units: 1,
                        attack_effort_points: 2,
                        cell,
                        motion,
                    })
                    .collect(),
            ))
    }
}
