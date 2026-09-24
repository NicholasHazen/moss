use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{ExecutorKind, ScheduleBuildSettings};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Id(u32);
#[derive(Component)]
struct Hunter { target: Id, cell: i32, reserve_units: u32, capacity_units: u32, alive: bool }
#[derive(Component)]
struct Prey { cell: i32, biomass_units: u32, available: bool, actions: u32 }
#[derive(Resource)]
struct Moment { run: u32, tick: u64 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Terminal { run: u32, tick: u64, prey: Id, hunter: Id, gained_units: u32 }
#[derive(Resource, Default)]
struct Journal(Vec<Terminal>);
#[derive(Resource, Default)]
struct Audit { pending: Vec<(Id, bool, u32, u32)>, acted: Vec<Id> }

fn claim(prey: &mut Prey, hunter: &mut Hunter) -> Option<u32> {
    if !hunter.alive || !prey.available || prey.biomass_units == 0
        || hunter.cell != prey.cell || hunter.reserve_units > hunter.capacity_units {
        return None;
    }
    let gain = prey.biomass_units;
    let room = hunter.capacity_units - hunter.reserve_units;
    if gain > room { return None; }
    let reserve = hunter.reserve_units.checked_add(gain).expect("gain fits validated room");
    prey.available = false;
    prey.biomass_units = 0;
    hunter.reserve_units = reserve;
    Some(gain)
}

fn resolve_hunts(
    mut commands: Commands,
    moment: Res<Moment>,
    mut hunters: Query<(Entity, &Id, &mut Hunter)>,
    mut prey_query: Query<(Entity, &Id, &mut Prey)>,
    mut journal: ResMut<Journal>,
) {
    let mut order: Vec<(Id, Entity)> = hunters.iter().map(|(entity, id, _)| (*id, entity)).collect();
    order.sort_by_key(|(id, _)| *id);
    for (_, entity) in order {
        let (_, hunter_id, mut hunter) = hunters.get_mut(entity).unwrap();
        for (prey_entity, prey_id, mut prey) in &mut prey_query {
            if *prey_id != hunter.target { continue; }
            if let Some(gained_units) = claim(&mut prey, &mut hunter) {
                journal.0.push(Terminal { run: moment.run, tick: moment.tick,
                    prey: *prey_id, hunter: *hunter_id, gained_units });
                commands.entity(prey_entity).despawn();
            }
            break;
        }
    }
}
fn prey_action(mut prey_query: Query<(&Id, &mut Prey)>, mut audit: ResMut<Audit>) {
    for (id, mut prey) in &mut prey_query {
        if !prey.available { continue; }
        prey.actions = prey.actions.checked_add(1).expect("action count exhausted");
        audit.acted.push(*id);
    }
}
fn observe_pending(prey_query: Query<(&Id, &Prey)>, mut audit: ResMut<Audit>) {
    audit.pending = prey_query.iter().map(|(id, prey)|
        (*id, prey.available, prey.biomass_units, prey.actions)).collect();
    audit.pending.sort_by_key(|row| row.0);
}
fn hunt_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.set_executor_kind(ExecutorKind::SingleThreaded);
    schedule.set_build_settings(ScheduleBuildSettings {
        auto_insert_apply_deferred: false, ..Default::default()
    });
    schedule.set_apply_final_deferred(false);
    schedule.add_systems((resolve_hunts, prey_action, observe_pending).chain());
    schedule
}
fn fixture(order: [u32; 2]) -> (World, Entity) {
    let mut world = World::new();
    world.insert_resource(Moment { run: 7, tick: 1 });
    world.init_resource::<Journal>();
    world.init_resource::<Audit>();
    for id in order {
        world.spawn((Id(id), Hunter { target: Id(40), cell: 0, reserve_units: 0,
            capacity_units: 10, alive: true }));
    }
    let prey = world.spawn((Id(40), Prey { cell: 0, biomass_units: 3,
        available: true, actions: 0 })).id();
    (world, prey)
}
fn reserves(world: &mut World) -> Vec<(Id, u32)> {
    let mut rows: Vec<_> = world.query::<(&Id, &Hunter)>().iter(world)
        .map(|(id, hunter)| (*id, hunter.reserve_units)).collect();
    rows.sort_by_key(|row| row.0);
    rows
}
fn main() {
    let (mut world, _) = fixture([9, 2]);
    let mut schedule = hunt_schedule();
    schedule.run(&mut world);
    println!("reserves={:?}", reserves(&mut world));
    println!("pending={:?}", world.resource::<Audit>().pending);
    println!("terminal records={}", world.resource::<Journal>().0.len());
    schedule.apply_deferred(&mut world);
    println!("prey after cleanup={}", world.query::<&Prey>().iter(&world).count());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn one_prey_rewards_one_hunter_before_structural_cleanup() {
        let (mut world, prey) = fixture([9, 2]);
        hunt_schedule().run(&mut world);
        assert_eq!(reserves(&mut world), vec![(Id(2), 3), (Id(9), 0)]);
        assert!(world.get::<Prey>(prey).is_some());
        assert_eq!(world.resource::<Audit>().pending, vec![(Id(40), false, 0, 0)]);
        assert!(world.resource::<Audit>().acted.is_empty());
        assert_eq!(world.resource::<Journal>().0, vec![Terminal {
            run: 7, tick: 1, prey: Id(40), hunter: Id(2), gained_units: 3 }]);
    }
    #[test]
    fn spawn_order_does_not_select_the_winner() {
        for order in [[9, 2], [2, 9]] {
            let (mut world, _) = fixture(order);
            hunt_schedule().run(&mut world);
            assert_eq!(reserves(&mut world), vec![(Id(2), 3), (Id(9), 0)]);
            assert_eq!(world.resource::<Journal>().0[0].hunter, Id(2));
        }
    }
    #[test]
    fn repeated_resolution_while_cleanup_is_pending_does_not_reward_again() {
        let (mut world, prey) = fixture([9, 2]);
        let mut schedule = hunt_schedule();
        schedule.run(&mut world);
        schedule.run(&mut world);
        assert!(world.get::<Prey>(prey).is_some());
        assert_eq!(reserves(&mut world), vec![(Id(2), 3), (Id(9), 0)]);
        assert_eq!(world.resource::<Journal>().0.len(), 1);
        assert!(world.resource::<Audit>().acted.is_empty());
        schedule.apply_deferred(&mut world);
        assert!(world.get::<Prey>(prey).is_none());
        schedule.run(&mut world);
        assert_eq!(world.resource::<Journal>().0.len(), 1);
    }
    #[test]
    fn insufficient_capacity_rejects_without_consumption_and_later_hunter_can_win() {
        let (mut world, _) = fixture([9, 2]);
        for (id, mut hunter) in world.query::<(&Id, &mut Hunter)>().iter_mut(&mut world) {
            if *id == Id(2) { hunter.reserve_units = 9; }
        }
        hunt_schedule().run(&mut world);
        assert_eq!(reserves(&mut world), vec![(Id(2), 9), (Id(9), 3)]);
        assert_eq!(world.resource::<Journal>().0[0].hunter, Id(9));
    }
    #[test]
    fn absent_contact_preserves_prey_and_allows_its_later_action() {
        let (mut world, prey) = fixture([9, 2]);
        world.get_mut::<Prey>(prey).unwrap().cell = 1;
        hunt_schedule().run(&mut world);
        assert_eq!(reserves(&mut world), vec![(Id(2), 0), (Id(9), 0)]);
        assert_eq!(world.resource::<Audit>().pending, vec![(Id(40), true, 3, 1)]);
        assert_eq!(world.resource::<Audit>().acted, vec![Id(40)]);
        assert!(world.resource::<Journal>().0.is_empty());
    }
    #[test]
    fn dead_hunter_and_invalid_reserve_cannot_claim_prey() {
        let mut prey = Prey { cell: 0, biomass_units: 3, available: true, actions: 0 };
        let mut hunter = Hunter { target: Id(40), cell: 0, reserve_units: 0,
            capacity_units: 10, alive: false };
        assert_eq!(claim(&mut prey, &mut hunter), None);
        hunter.alive = true;
        hunter.reserve_units = 11;
        assert_eq!(claim(&mut prey, &mut hunter), None);
        assert_eq!((prey.available, prey.biomass_units, hunter.reserve_units), (true, 3, 11));
    }
    #[test]
    fn exact_capacity_at_integer_limit_is_safe_and_zero_biomass_is_ineligible() {
        let mut prey = Prey { cell: 0, biomass_units: 3, available: true, actions: 0 };
        let mut hunter = Hunter { target: Id(40), cell: 0, reserve_units: u32::MAX - 3,
            capacity_units: u32::MAX, alive: true };
        assert_eq!(claim(&mut prey, &mut hunter), Some(3));
        assert_eq!(hunter.reserve_units, u32::MAX);
        prey.available = true;
        hunter.reserve_units = 0;
        assert_eq!(claim(&mut prey, &mut hunter), None);
        assert_eq!(hunter.reserve_units, 0);
    }
    #[test]
    fn a_missing_target_grants_no_reward_or_terminal_record() {
        let (mut world, prey) = fixture([9, 2]);
        assert!(world.despawn(prey));
        hunt_schedule().run(&mut world);
        assert_eq!(reserves(&mut world), vec![(Id(2), 0), (Id(9), 0)]);
        assert!(world.resource::<Journal>().0.is_empty());
        assert!(world.resource::<Audit>().acted.is_empty());
    }

}
