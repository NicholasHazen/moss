use bevy_ecs::{
    prelude::*,
    schedule::{ApplyDeferred, ExecutorKind},
};
use std::collections::{BTreeMap, BTreeSet};

const PARENT_COST: u32 = 3;
const CHILD_RESERVE: u32 = 4;
const COOLDOWN_TICKS: u64 = 3;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
struct Animal {
    id: u64,
    reserve: u32,
    adult: bool,
    cooldown_until: u64,
    first_eligible_tick: u64,
    parents: Option<[u64; 2]>,
    actions: u32,
}

fn adult(id: u64, reserve: u32) -> Animal {
    Animal {
        id,
        reserve,
        adult: true,
        cooldown_until: 0,
        first_eligible_tick: 1,
        parents: None,
        actions: 0,
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Rejected {
    SameParent,
    Ineligible,
    Unaffordable,
    InvalidChildId,
    CounterOverflow,
}

#[derive(Debug, PartialEq, Eq)]
struct BirthPlan {
    a: Animal,
    b: Animal,
    child: Animal,
    next_id: u64,
}

fn plan_birth(
    mut a: Animal,
    mut b: Animal,
    tick: u64,
    child_id: u64,
) -> Result<BirthPlan, Rejected> {
    if a.id == b.id {
        return Err(Rejected::SameParent);
    }
    if !a.adult
        || !b.adult
        || a.first_eligible_tick > tick
        || b.first_eligible_tick > tick
        || a.cooldown_until > tick
        || b.cooldown_until > tick
    {
        return Err(Rejected::Ineligible);
    }
    if a.reserve < PARENT_COST || b.reserve < PARENT_COST {
        return Err(Rejected::Unaffordable);
    }
    if child_id == 0 || child_id == a.id || child_id == b.id {
        return Err(Rejected::InvalidChildId);
    }
    let cooldown_until = tick
        .checked_add(COOLDOWN_TICKS)
        .ok_or(Rejected::CounterOverflow)?;
    let first_eligible_tick = tick.checked_add(1).ok_or(Rejected::CounterOverflow)?;
    let next_id = child_id.checked_add(1).ok_or(Rejected::CounterOverflow)?;
    a.reserve -= PARENT_COST;
    b.reserve -= PARENT_COST;
    a.cooldown_until = cooldown_until;
    b.cooldown_until = cooldown_until;
    let child = Animal {
        id: child_id,
        reserve: CHILD_RESERVE,
        adult: false,
        cooldown_until: 0,
        first_eligible_tick,
        parents: Some([a.id.min(b.id), a.id.max(b.id)]),
        actions: 0,
    };
    Ok(BirthPlan {
        a,
        b,
        child,
        next_id,
    })
}

#[derive(Resource, Default)]
struct Tick(u64);
#[derive(Resource)]
struct NextId(u64);
#[derive(Resource, Default)]
struct Requests(Vec<(u64, u64)>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Birth {
    tick: u64,
    child: u64,
    parents: [u64; 2],
}
#[derive(Resource, Default)]
struct LastBirths(Vec<Birth>);

fn begin_tick(mut tick: ResMut<Tick>) {
    tick.0 = tick.0.checked_add(1).expect("tick counter overflow");
}

fn resolve_births(
    mut commands: Commands,
    tick: Res<Tick>,
    mut next: ResMut<NextId>,
    mut requests: ResMut<Requests>,
    mut births: ResMut<LastBirths>,
    mut animals: Query<(Entity, &mut Animal)>,
) {
    births.0.clear();
    let ids: BTreeMap<_, _> = animals
        .iter()
        .map(|(entity, animal)| (animal.id, entity))
        .collect();
    let pairs: BTreeSet<_> = std::mem::take(&mut requests.0)
        .into_iter()
        .map(|(a, b)| (a.min(b), a.max(b)))
        .collect();
    for (a, b) in pairs {
        let (Some(&a_entity), Some(&b_entity)) = (ids.get(&a), ids.get(&b)) else {
            continue;
        };
        if ids.contains_key(&next.0) {
            continue;
        }
        let Ok([(_, mut left), (_, mut right)]) = animals.get_many_mut([a_entity, b_entity]) else {
            continue;
        };
        let Ok(plan) = plan_birth(*left, *right, tick.0, next.0) else {
            continue;
        };
        *left = plan.a;
        *right = plan.b;
        next.0 = plan.next_id;
        commands.spawn(plan.child);
        births.0.push(Birth {
            tick: tick.0,
            child: plan.child.id,
            parents: plan.child.parents.expect("a planned child has parentage"),
        });
    }
}

fn act(tick: Res<Tick>, mut animals: Query<&mut Animal>) {
    for mut animal in &mut animals {
        if tick.0 >= animal.first_eligible_tick {
            animal.actions = animal
                .actions
                .checked_add(1)
                .expect("action counter overflow");
        }
    }
}

fn schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.add_systems((begin_tick, resolve_births, ApplyDeferred, act).chain());
    schedule
}

fn fixture(requests: Vec<(u64, u64)>) -> World {
    let mut world = World::new();
    world.init_resource::<Tick>();
    world.insert_resource(NextId(100));
    world.insert_resource(Requests(requests));
    world.init_resource::<LastBirths>();
    world.spawn(adult(1, 10));
    world.spawn(adult(2, 10));
    world
}

fn readings(world: &mut World) -> Vec<Animal> {
    let mut rows: Vec<_> = world.query::<&Animal>().iter(world).copied().collect();
    rows.sort_by_key(|animal| animal.id);
    rows
}

fn main() {
    let mut world = fixture(vec![(2, 1), (1, 2)]);
    let mut schedule = schedule();
    for _ in 0..2 {
        schedule.run(&mut world);
        println!(
            "tick={} births={}",
            world.resource::<Tick>().0,
            world.resource::<LastBirths>().0.len()
        );
        for animal in readings(&mut world) {
            println!(
                "  #{} reserve={} cooldown={} eligible={} actions={} parents={:?}",
                animal.id,
                animal.reserve,
                animal.cooldown_until,
                animal.first_eligible_tick,
                animal.actions,
                animal.parents
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_parents_pay_and_receive_the_same_deadline() {
        let exact = plan_birth(adult(1, PARENT_COST), adult(2, PARENT_COST), 1, 100).unwrap();
        assert_eq!((exact.a.reserve, exact.b.reserve), (0, 0));
        let plan = plan_birth(adult(2, 10), adult(1, 8), 1, 100).unwrap();
        assert_eq!((plan.a.reserve, plan.b.reserve), (7, 5));
        assert_eq!((plan.a.cooldown_until, plan.b.cooldown_until), (4, 4));
        assert_eq!(plan.child.parents, Some([1, 2]));
        assert_eq!(
            (
                plan.child.reserve,
                plan.child.first_eligible_tick,
                plan.next_id
            ),
            (4, 2, 101)
        );
    }

    #[test]
    fn rejected_pair_cannot_partly_change_the_callers() {
        let a = adult(1, 10);
        let b = adult(2, 2);
        assert_eq!(plan_birth(a, b, 1, 100), Err(Rejected::Unaffordable));
        assert_eq!(
            (a.reserve, b.reserve, a.cooldown_until, b.cooldown_until),
            (10, 2, 0, 0)
        );
        let mut world = fixture(vec![(1, 2)]);
        world
            .query::<&mut Animal>()
            .iter_mut(&mut world)
            .find(|a| a.id == 2)
            .unwrap()
            .reserve = 2;
        schedule().run(&mut world);
        let rows = readings(&mut world);
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].reserve, rows[1].reserve), (10, 2));
        assert_eq!(world.resource::<NextId>().0, 100);
        assert!(world.resource::<LastBirths>().0.is_empty());
    }

    #[test]
    fn cooldown_boundary_is_inclusive_and_immature_parents_are_rejected() {
        let later = Animal {
            first_eligible_tick: 5,
            ..adult(1, 10)
        };
        assert_eq!(
            plan_birth(later, adult(2, 10), 4, 100),
            Err(Rejected::Ineligible)
        );
        assert!(plan_birth(later, adult(2, 10), 5, 100).is_ok());
        let first = plan_birth(adult(1, 10), adult(2, 10), 1, 100).unwrap();
        assert_eq!(
            plan_birth(first.a, first.b, 3, 101),
            Err(Rejected::Ineligible)
        );
        assert!(plan_birth(first.a, first.b, 4, 101).is_ok());
        assert_eq!(
            plan_birth(first.a, first.child, 4, 101),
            Err(Rejected::Ineligible)
        );
    }

    #[test]
    fn invalid_identity_and_counter_limits_reject_without_a_plan() {
        let a = adult(1, 10);
        let b = adult(2, 10);
        assert_eq!(plan_birth(a, a, 1, 100), Err(Rejected::SameParent));
        for id in [0, 1, 2] {
            assert_eq!(plan_birth(a, b, 1, id), Err(Rejected::InvalidChildId));
        }
        assert_eq!(
            plan_birth(a, b, u64::MAX - 1, 100),
            Err(Rejected::CounterOverflow)
        );
        assert_eq!(
            plan_birth(a, b, 1, u64::MAX),
            Err(Rejected::CounterOverflow)
        );
    }

    #[test]
    fn duplicate_unordered_requests_produce_one_child_and_one_charge() {
        let mut world = fixture(vec![(2, 1), (1, 2), (2, 1)]);
        schedule().run(&mut world);
        let rows = readings(&mut world);
        assert_eq!(rows.len(), 3);
        assert_eq!((rows[0].reserve, rows[1].reserve), (7, 7));
        assert_eq!(
            world.resource::<LastBirths>().0,
            vec![Birth {
                tick: 1,
                child: 100,
                parents: [1, 2]
            }]
        );
        assert_eq!(world.resource::<NextId>().0, 101);
    }

    #[test]
    fn flushed_newborn_is_present_but_cannot_act_until_the_next_tick() {
        let mut world = fixture(vec![(1, 2)]);
        let mut schedule = schedule();
        schedule.run(&mut world);
        let child = readings(&mut world)
            .into_iter()
            .find(|a| a.id == 100)
            .unwrap();
        assert_eq!((child.first_eligible_tick, child.actions), (2, 0));
        schedule.run(&mut world);
        let child = readings(&mut world)
            .into_iter()
            .find(|a| a.id == 100)
            .unwrap();
        assert_eq!(child.actions, 1);
        assert_eq!(child.parents, Some([1, 2]));
    }

    #[test]
    fn shared_parent_is_claimed_in_canonical_pair_order() {
        let mut world = fixture(vec![(3, 1), (2, 1)]);
        world.spawn(adult(3, 10));
        schedule().run(&mut world);
        assert_eq!(world.resource::<LastBirths>().0[0].parents, [1, 2]);
        let rows = readings(&mut world);
        assert_eq!(rows.iter().find(|a| a.id == 3).unwrap().reserve, 10);
        assert_eq!(rows.len(), 4);
    }

    #[test]
    fn missing_parent_or_colliding_child_id_leaves_reserves_unchanged() {
        for (requests, next_id) in [(vec![(1, 99)], 100), (vec![(1, 2)], 2)] {
            let mut world = fixture(requests);
            world.resource_mut::<NextId>().0 = next_id;
            schedule().run(&mut world);
            let rows = readings(&mut world);
            assert_eq!(rows.len(), 2);
            assert_eq!((rows[0].reserve, rows[1].reserve), (10, 10));
            assert!(world.resource::<LastBirths>().0.is_empty());
        }
    }
}
