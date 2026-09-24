use bevy_ecs::{prelude::*, schedule::ExecutorKind};

#[derive(Resource)]
struct Counter(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase { Before, After }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Observation {
    phase: Phase,
    count: u32,
}

#[derive(Resource, Default)]
struct Trace(Vec<Observation>);

fn record_before(counter: Res<Counter>, mut trace: ResMut<Trace>) {
    trace.0.push(Observation { phase: Phase::Before, count: counter.0 });
}

fn increment(mut counter: ResMut<Counter>) {
    counter.0 = counter.0.checked_add(1).expect("counter overflow");
}

fn record_after(counter: Res<Counter>, mut trace: ResMut<Trace>) {
    trace.0.push(Observation { phase: Phase::After, count: counter.0 });
}

fn observation_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems((record_before, increment, record_after).chain());
    schedule
}

fn world_at(count: u32) -> World {
    let mut world = World::new();
    world.insert_resource(Counter(count));
    world.init_resource::<Trace>();
    world
}

fn main() {
    let mut world = world_at(7);
    let mut schedule = observation_schedule();
    schedule.run(&mut world);
    schedule.run(&mut world);
    for observation in &world.resource::<Trace>().0 {
        println!("{:?} {}", observation.phase, observation.count);
    }
    println!("final={}", world.resource::<Counter>().0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observations_bracket_one_real_mutation() {
        let mut world = world_at(7);
        observation_schedule().run(&mut world);
        assert_eq!(world.resource::<Trace>().0, vec![
            Observation { phase: Phase::Before, count: 7 },
            Observation { phase: Phase::After, count: 8 },
        ]);
        assert_eq!(world.resource::<Counter>().0, 8);
    }

    #[test]
    fn successive_runs_append_fresh_observations_in_execution_order() {
        let mut world = world_at(41);
        let mut schedule = observation_schedule();
        schedule.run(&mut world);
        schedule.run(&mut world);
        assert_eq!(world.resource::<Trace>().0, vec![
            Observation { phase: Phase::Before, count: 41 },
            Observation { phase: Phase::After, count: 42 },
            Observation { phase: Phase::Before, count: 42 },
            Observation { phase: Phase::After, count: 43 },
        ]);
        assert_eq!(world.resource::<Counter>().0, 43);
    }

    #[test]
    fn same_final_counter_can_hide_a_late_before_observation() {
        let mut correct = world_at(7);
        observation_schedule().run(&mut correct);
        let mut wrong = world_at(7);
        let mut schedule = Schedule::default();
        schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        schedule.add_systems((increment, record_before, record_after).chain());
        schedule.run(&mut wrong);
        assert_eq!(wrong.resource::<Counter>().0, correct.resource::<Counter>().0);
        assert_eq!(wrong.resource::<Trace>().0, vec![
            Observation { phase: Phase::Before, count: 8 },
            Observation { phase: Phase::After, count: 8 },
        ]);
        assert_ne!(wrong.resource::<Trace>().0, correct.resource::<Trace>().0);
    }

    #[test]
    fn same_final_counter_can_hide_an_early_after_observation() {
        let mut correct = world_at(7);
        observation_schedule().run(&mut correct);
        let mut wrong = world_at(7);
        let mut schedule = Schedule::default();
        schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        schedule.add_systems((record_before, record_after, increment).chain());
        schedule.run(&mut wrong);
        assert_eq!(wrong.resource::<Counter>().0, correct.resource::<Counter>().0);
        assert_eq!(wrong.resource::<Trace>().0, vec![
            Observation { phase: Phase::Before, count: 7 },
            Observation { phase: Phase::After, count: 7 },
        ]);
        assert_ne!(wrong.resource::<Trace>().0, correct.resource::<Trace>().0);
    }
}
