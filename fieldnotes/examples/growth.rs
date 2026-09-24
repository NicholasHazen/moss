use bevy_ecs::{prelude::*, schedule::ExecutorKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InvalidSetup {
    OverCapacity,
    InvalidDay,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
struct Patch {
    biomass_units: u32,
    capacity_units: u32,
}

impl Patch {
    fn new(biomass_units: u32, capacity_units: u32) -> Result<Self, InvalidSetup> {
        if biomass_units > capacity_units {
            return Err(InvalidSetup::OverCapacity);
        }
        Ok(Self {
            biomass_units,
            capacity_units,
        })
    }
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
struct Daylight {
    cycle_ticks: u64,
    lit_ticks: u64,
}

impl Daylight {
    fn new(cycle_ticks: u64, lit_ticks: u64) -> Result<Self, InvalidSetup> {
        if cycle_ticks == 0 || lit_ticks > cycle_ticks {
            return Err(InvalidSetup::InvalidDay);
        }
        Ok(Self {
            cycle_ticks,
            lit_ticks,
        })
    }

    fn is_lit(self, tick: u64) -> bool {
        let zero_based = tick.checked_sub(1).expect("executed ticks begin at one");
        zero_based % self.cycle_ticks < self.lit_ticks
    }
}

#[derive(Resource, Default)]
struct Tick(u64);

#[derive(Resource)]
struct Rules {
    growth_units_per_lit_tick: u32,
    meal_units_per_tick: u32,
}

#[derive(Resource, Default)]
struct Grazer {
    reserve_units: u32,
}

const RESERVE_CAPACITY_UNITS: u32 = 20;

#[derive(Resource, Default)]
struct Flow {
    added_units: u32,
    eaten_units: u32,
}

fn renew(patch: &mut Patch, lit: bool, requested_units: u32) -> u32 {
    if !lit {
        return 0;
    }
    let space = patch.capacity_units - patch.biomass_units;
    let added = requested_units.min(space);
    patch.biomass_units += added;
    added
}

fn begin_tick(mut tick: ResMut<Tick>) {
    tick.0 = tick.0.checked_add(1).expect("tick counter exhausted");
}

fn grow(
    tick: Res<Tick>,
    daylight: Res<Daylight>,
    rules: Res<Rules>,
    mut patch: ResMut<Patch>,
    mut flow: ResMut<Flow>,
) {
    flow.added_units = renew(
        &mut patch,
        daylight.is_lit(tick.0),
        rules.growth_units_per_lit_tick,
    );
}

fn meal(
    rules: Res<Rules>,
    mut patch: ResMut<Patch>,
    mut grazer: ResMut<Grazer>,
    mut flow: ResMut<Flow>,
) {
    let space = RESERVE_CAPACITY_UNITS - grazer.reserve_units;
    let eaten = rules
        .meal_units_per_tick
        .min(patch.biomass_units)
        .min(space);
    patch.biomass_units -= eaten;
    grazer.reserve_units += eaten;
    flow.eaten_units = eaten;
}

#[derive(Clone, Copy, Debug)]
enum Order {
    GrowthBeforeMeal,
    GrowthAfterMeal,
}

fn tick_schedule(order: Order) -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    match order {
        Order::GrowthBeforeMeal => {
            schedule.add_systems((begin_tick, grow, meal).chain());
        }
        Order::GrowthAfterMeal => {
            schedule.add_systems((begin_tick, meal, grow).chain());
        }
    }
    schedule
}

fn fixture(biomass: u32, capacity: u32, growth: u32, bite: u32) -> World {
    let mut world = World::new();
    world.insert_resource(Patch::new(biomass, capacity).unwrap());
    world.insert_resource(Daylight::new(4, 2).unwrap());
    world.insert_resource(Rules {
        growth_units_per_lit_tick: growth,
        meal_units_per_tick: bite,
    });
    world.init_resource::<Tick>();
    world.init_resource::<Grazer>();
    world.init_resource::<Flow>();
    world
}

#[derive(Debug, PartialEq, Eq)]
struct Reading {
    tick: u64,
    biomass: u32,
    reserve: u32,
    added: u32,
    eaten: u32,
}

fn inspect(world: &World) -> Reading {
    Reading {
        tick: world.resource::<Tick>().0,
        biomass: world.resource::<Patch>().biomass_units,
        reserve: world.resource::<Grazer>().reserve_units,
        added: world.resource::<Flow>().added_units,
        eaten: world.resource::<Flow>().eaten_units,
    }
}

fn main() {
    for order in [Order::GrowthBeforeMeal, Order::GrowthAfterMeal] {
        let mut world = fixture(0, 4, 3, 2);
        let mut schedule = tick_schedule(order);
        println!("{order:?}");
        for _ in 0..4 {
            schedule.run(&mut world);
            println!("{:?}", inspect(&world));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_boundaries_follow_executed_ticks() {
        let day = Daylight::new(4, 2).unwrap();
        let phases: Vec<_> = (1..=9).map(|tick| day.is_lit(tick)).collect();
        assert_eq!(
            phases,
            [true, true, false, false, true, true, false, false, true]
        );
        assert!(!Daylight::new(4, 0).unwrap().is_lit(1));
        assert!(Daylight::new(4, 4).unwrap().is_lit(4));
    }

    #[test]
    fn invalid_inputs_are_rejected_before_a_world_exists() {
        assert_eq!(Daylight::new(0, 0), Err(InvalidSetup::InvalidDay));
        assert_eq!(Daylight::new(4, 5), Err(InvalidSetup::InvalidDay));
        assert_eq!(Patch::new(5, 4), Err(InvalidSetup::OverCapacity));
    }

    #[test]
    fn capacity_limits_actual_supply_even_near_the_integer_limit() {
        let mut patch = Patch::new(u32::MAX - 1, u32::MAX).unwrap();
        assert_eq!(renew(&mut patch, true, u32::MAX), 1);
        assert_eq!(patch.biomass_units, u32::MAX);
        assert_eq!(renew(&mut patch, true, 8), 0);
        let mut closed = Patch::new(0, 0).unwrap();
        assert_eq!(renew(&mut closed, true, 8), 0);
    }

    #[test]
    fn darkness_preserves_biomass_and_reports_no_supply() {
        let mut patch = Patch::new(2, 4).unwrap();
        assert_eq!(renew(&mut patch, false, 3), 0);
        assert_eq!(patch.biomass_units, 2);
    }

    #[test]
    fn an_empty_patch_can_feed_this_tick_only_when_growth_runs_first() {
        let mut before = fixture(0, 4, 3, 2);
        tick_schedule(Order::GrowthBeforeMeal).run(&mut before);
        assert_eq!(
            inspect(&before),
            Reading {
                tick: 1,
                biomass: 1,
                reserve: 2,
                added: 3,
                eaten: 2
            }
        );
        let mut after = fixture(0, 4, 3, 2);
        tick_schedule(Order::GrowthAfterMeal).run(&mut after);
        assert_eq!(
            inspect(&after),
            Reading {
                tick: 1,
                biomass: 3,
                reserve: 0,
                added: 3,
                eaten: 0
            }
        );
    }

    #[test]
    fn meal_first_can_open_capacity_for_extra_external_supply() {
        let mut before = fixture(4, 4, 3, 2);
        tick_schedule(Order::GrowthBeforeMeal).run(&mut before);
        assert_eq!(
            inspect(&before),
            Reading {
                tick: 1,
                biomass: 2,
                reserve: 2,
                added: 0,
                eaten: 2
            }
        );
        let mut after = fixture(4, 4, 3, 2);
        tick_schedule(Order::GrowthAfterMeal).run(&mut after);
        assert_eq!(
            inspect(&after),
            Reading {
                tick: 1,
                biomass: 4,
                reserve: 2,
                added: 2,
                eaten: 2
            }
        );
    }

    #[test]
    fn installed_growth_uses_light_boundaries_and_reports_actual_additions() {
        let mut world = fixture(0, 4, 3, 0);
        let mut schedule = tick_schedule(Order::GrowthBeforeMeal);
        let mut trace = Vec::new();
        for _ in 0..5 {
            schedule.run(&mut world);
            let reading = inspect(&world);
            trace.push((reading.biomass, reading.added));
        }
        assert_eq!(trace, [(3, 3), (4, 1), (4, 0), (4, 0), (4, 0)]);
        assert_eq!(world.resource::<Tick>().0, 5);
    }

    #[test]
    fn meal_transfer_and_external_supply_have_separate_accounts() {
        let mut world = fixture(4, 4, 3, 2);
        let mut schedule = tick_schedule(Order::GrowthAfterMeal);
        let mut previous_total = 4_u64;
        for _ in 0..9 {
            schedule.run(&mut world);
            let reading = inspect(&world);
            let total = u64::from(reading.biomass) + u64::from(reading.reserve);
            assert_eq!(total, previous_total + u64::from(reading.added));
            previous_total = total;
        }
    }

    #[test]
    fn full_reserve_does_not_remove_a_patchs_food() {
        let mut world = fixture(4, 4, 0, u32::MAX);
        world.resource_mut::<Grazer>().reserve_units = RESERVE_CAPACITY_UNITS;
        tick_schedule(Order::GrowthBeforeMeal).run(&mut world);
        assert_eq!(
            inspect(&world),
            Reading {
                tick: 1,
                biomass: 4,
                reserve: 20,
                added: 0,
                eaten: 0
            }
        );
    }

    #[test]
    fn extra_inspections_do_not_advance_or_feed_the_world() {
        let mut sparse = fixture(0, 4, 3, 2);
        let mut busy = fixture(0, 4, 3, 2);
        let mut sparse_schedule = tick_schedule(Order::GrowthBeforeMeal);
        let mut busy_schedule = tick_schedule(Order::GrowthBeforeMeal);
        for _ in 0..7 {
            sparse_schedule.run(&mut sparse);
            let before_reads = inspect(&busy);
            for _ in 0..100 {
                assert_eq!(inspect(&busy), before_reads);
            }
            busy_schedule.run(&mut busy);
            assert_eq!(inspect(&busy), inspect(&sparse));
        }
    }
}
