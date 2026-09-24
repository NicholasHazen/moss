//! Fully declared course fixtures, shared by native examples and preview callers.

use super::{
    Cell, GrazerPlacement, GridBounds, MobileScenario, Motion, PatchPlacement, SpatialConfig,
};
use crate::{
    Daylight, FounderTrait, GrazerSeed, PatchSeed, PopulationConfig, Scenario, SimId, UpkeepTrait,
};

impl MobileScenario {
    /// One grazer: two paid steps to three finite units, with no renewal.
    pub fn short_journey() -> Self {
        let motion = Motion {
            sensing_radius: 3,
            max_cells_per_tick: 1,
            travel_units_per_cell: 1,
        };
        Self::new(
            Scenario {
                daylight: Daylight::new(1, 0).unwrap(),
                grazers: vec![GrazerSeed {
                    id: SimId(1),
                    reserve_units: 5,
                    capacity_units: 8,
                    maintenance_units_per_tick: 1,
                    meal_units_per_tick: 2,
                    feeding_site: None,
                }],
                patches: vec![PatchSeed {
                    id: SimId(100),
                    biomass_units: 3,
                    capacity_units: 3,
                    growth_units_per_lit_tick: 0,
                }],
                history_limit: 128,
            },
            PopulationConfig::new(vec![FounderTrait {
                id: SimId(1),
                upkeep: UpkeepTrait::new(1).unwrap(),
            }]),
            SpatialConfig {
                bounds: GridBounds {
                    width_cells: 4,
                    height_cells: 2,
                },
                grazers: vec![GrazerPlacement {
                    id: SimId(1),
                    cell: Cell { x: 0, y: 0 },
                    motion,
                }],
                patches: vec![PatchPlacement {
                    id: SimId(100),
                    cell: Cell { x: 2, y: 0 },
                }],
                newborn_motion: motion,
            },
        )
    }

    /// Four founders, two renewable patches, inherited upkeep, and a grazer cap10.
    pub fn meadow() -> Self {
        let cells = [
            Cell { x: 2, y: 2 },
            Cell { x: 3, y: 1 },
            Cell { x: 2, y: 3 },
            Cell { x: 3, y: 2 },
        ];
        let founders: Vec<_> = [1, 1, 2, 2]
            .into_iter()
            .enumerate()
            .map(|(index, value)| FounderTrait {
                id: SimId(index as u32 + 1),
                upkeep: UpkeepTrait::new(value).unwrap(),
            })
            .collect();
        let motion = Motion {
            sensing_radius: 5,
            max_cells_per_tick: 1,
            travel_units_per_cell: 1,
        };
        let scenario = Scenario {
            daylight: Daylight::new(8, 4).unwrap(),
            grazers: founders
                .iter()
                .map(|founder| GrazerSeed {
                    id: founder.id,
                    reserve_units: 18,
                    capacity_units: 24,
                    maintenance_units_per_tick: founder.upkeep.units(),
                    meal_units_per_tick: 5,
                    feeding_site: None,
                })
                .collect(),
            patches: vec![
                PatchSeed {
                    id: SimId(1000),
                    biomass_units: 6,
                    capacity_units: 18,
                    growth_units_per_lit_tick: 8,
                },
                PatchSeed {
                    id: SimId(1001),
                    biomass_units: 18,
                    capacity_units: 18,
                    growth_units_per_lit_tick: 4,
                },
            ],
            history_limit: 128,
        };
        let space = SpatialConfig {
            bounds: GridBounds {
                width_cells: 11,
                height_cells: 5,
            },
            grazers: founders
                .iter()
                .zip(cells)
                .map(|(founder, cell)| GrazerPlacement {
                    id: founder.id,
                    cell,
                    motion,
                })
                .collect(),
            patches: vec![
                PatchPlacement {
                    id: SimId(1000),
                    cell: Cell { x: 3, y: 2 },
                },
                PatchPlacement {
                    id: SimId(1001),
                    cell: Cell { x: 7, y: 2 },
                },
            ],
            newborn_motion: motion,
        };
        let mut population = PopulationConfig::new(founders);
        population.max_living = 10;
        Self::new(scenario, population, space)
    }
}
