// Presentation adapter for the exact saved course crate. All ecological rules
// remain in that crate. Reads return its last owned snapshot without stepping.
use std::cell::RefCell;

use course::{CourseWorld, EventKind, Life, Scenario, Snapshot};

struct BrowserWorld {
    simulation: CourseWorld,
    observation: Snapshot,
}

impl BrowserWorld {
    fn new() -> Self {
        let simulation =
            CourseWorld::new(Scenario::limited_supply()).expect("valid course fixture");
        let observation = simulation.snapshot();
        Self {
            simulation,
            observation,
        }
    }
}

thread_local! {
    static HOST: RefCell<BrowserWorld> = RefCell::new(BrowserWorld::new());
}

// SAFETY: The unique scalar-only exports below comprise this isolated host's
// complete C ABI. No caller-owned addresses or shared mutable globals cross it.
#[unsafe(no_mangle)]
pub extern "C" fn moss_course_reset(scenario: u32) -> u32 {
    let scenario = match scenario {
        0 => Scenario::limited_supply(),
        1 => Scenario::generous_supply(),
        _ => return 0,
    };
    HOST.with(|host| {
        let mut host = host.borrow_mut();
        if host.simulation.reset_with(scenario).is_err() {
            return 0;
        }
        host.observation = host.simulation.snapshot();
        1
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn moss_course_step() {
    HOST.with(|host| {
        let mut host = host.borrow_mut();
        host.simulation.step();
        host.observation = host.simulation.snapshot();
    });
}

/// Tables: 0 summary, 1 grazers, 2 patches, 3 retained events.
/// Unknown addresses yield u64::MAX. That bit pattern is also a valid u64 value;
/// use moss_course_valid to distinguish it. Optional IDs have a separate
/// presence column, so the valid stable ID zero never means "missing".
#[unsafe(no_mangle)]
pub extern "C" fn moss_course_read(table: u32, row: u32, column: u32) -> u64 {
    HOST.with(|host| {
        let host = host.borrow();
        let s = &host.observation;
        let row = row as usize;
        let value = match table {
            0 if row == 0 => [
                s.run,
                s.tick,
                match s.lit {
                    None => 0,
                    Some(false) => 1,
                    Some(true) => 2,
                },
                s.grazers.len() as u64,
                s.patches.len() as u64,
                s.history.events.len() as u64,
                s.ledger.added_units,
                s.ledger.maintenance_units,
                s.ledger.eaten_units,
                s.ledger.starvations,
                s.history.complete_after_tick,
                s.history.collected_through_tick,
                s.history.evicted_events,
                s.history.limit as u64,
                s.daylight.cycle_ticks(),
                s.daylight.lit_ticks(),
            ]
            .get(column as usize)
            .copied(),
            1 => s.grazers.get(row).and_then(|g| {
                [
                    u64::from(g.id.0),
                    u64::from(g.reserve_units),
                    u64::from(g.capacity_units),
                    u64::from(g.maintenance_units_per_tick),
                    u64::from(g.meal_units_per_tick),
                    u64::from(g.life == Life::Alive),
                    u64::from(g.feeding_site.is_some()),
                    g.feeding_site.map_or(0, |id| u64::from(id.0)),
                ]
                .get(column as usize)
                .copied()
            }),
            2 => s.patches.get(row).and_then(|p| {
                [
                    u64::from(p.id.0),
                    u64::from(p.biomass_units),
                    u64::from(p.capacity_units),
                    u64::from(p.growth_units_per_lit_tick),
                ]
                .get(column as usize)
                .copied()
            }),
            3 => s.history.events.get(row).and_then(|event| {
                let (kind, actor, target, units) = match event.kind {
                    EventKind::Growth { patch, units } => (0, patch.0, 0, units),
                    EventKind::Maintenance { grazer, units } => (1, grazer.0, 0, units),
                    EventKind::Meal {
                        grazer,
                        patch,
                        units,
                    } => (2, grazer.0, patch.0, units),
                    EventKind::Starved { grazer } => (3, grazer.0, 0, 0),
                };
                [
                    event.run,
                    event.tick,
                    kind,
                    u64::from(actor),
                    u64::from(target),
                    u64::from(units),
                ]
                .get(column as usize)
                .copied()
            }),
            _ => None,
        };
        value.unwrap_or(u64::MAX)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn moss_course_valid(table: u32, row: u32, column: u32) -> u32 {
    HOST.with(|host| {
        let host = host.borrow();
        let s = &host.observation;
        u32::from(match table {
            0 => row == 0 && column < 16,
            1 => (row as usize) < s.grazers.len() && column < 8,
            2 => (row as usize) < s.patches.len() && column < 4,
            3 => (row as usize) < s.history.events.len() && column < 6,
            _ => false,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_never_step_and_reset_keeps_simulation_run_identity() {
        assert_eq!(moss_course_read(0, 0, 1), 0);
        moss_course_step();
        assert_eq!(moss_course_read(0, 0, 1), 1);
        assert_eq!(moss_course_read(0, 0, 1), 1);
        let run = moss_course_read(0, 0, 0);
        assert_eq!(moss_course_reset(99), 0);
        assert_eq!(moss_course_read(0, 0, 0), run);
        assert_eq!(moss_course_reset(1), 1);
        assert_eq!(moss_course_read(0, 0, 0), run + 1);
        assert_eq!(moss_course_read(0, 0, 1), 0);
    }

    #[test]
    fn published_rows_match_the_simulations_owned_snapshot() {
        moss_course_step();
        HOST.with(|host| {
            let snapshot = host.borrow().simulation.snapshot();
            assert_eq!(
                moss_course_read(1, 0, 1),
                u64::from(snapshot.grazers[0].reserve_units)
            );
            assert_eq!(
                moss_course_read(2, 0, 1),
                u64::from(snapshot.patches[0].biomass_units)
            );
            assert_eq!(
                moss_course_read(0, 0, 5),
                snapshot.history.events.len() as u64
            );
        });
        assert_eq!(moss_course_read(1, u32::MAX, 0), u64::MAX);
        assert_eq!(moss_course_read(0, 99, 1), u64::MAX);
        assert_eq!(moss_course_valid(0, 99, 1), 0);
    }

    #[test]
    fn a_valid_maximum_value_is_distinct_from_an_invalid_address() {
        HOST.with(|host| {
            let mut scenario = Scenario::limited_supply();
            scenario.daylight = course::Daylight::new(u64::MAX, u64::MAX).unwrap();
            let simulation = CourseWorld::new(scenario).unwrap();
            let observation = simulation.snapshot();
            *host.borrow_mut() = BrowserWorld {
                simulation,
                observation,
            };
        });
        assert_eq!(moss_course_valid(0, 0, 14), 1);
        assert_eq!(moss_course_read(0, 0, 14), u64::MAX);
        assert_eq!(moss_course_valid(0, 0, 16), 0);
    }
}
