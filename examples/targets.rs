#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Id(u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position { x: i32, y: i32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Patch { id: Id, position: Position, biomass_units: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PatchReading { id: Id, position: Position, biomass_units: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Actor {
    position: Position,
    reserve_units: u32,
    capacity_units: u32,
    target: Option<Id>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Rejection { InvalidReserve, NoTarget, MissingTarget, EmptyTarget, NotInContact, NoRoom }

fn distance(a: Position, b: Position) -> u64 {
    u64::from(a.x.abs_diff(b.x)) + u64::from(a.y.abs_diff(b.y))
}

fn observe_local(position: Position, radius: u32, patches: &[Patch]) -> Vec<PatchReading> {
    patches.iter().filter(|patch| distance(position, patch.position) <= u64::from(radius))
        .map(|patch| PatchReading {
            id: patch.id, position: patch.position, biomass_units: patch.biomass_units,
        }).collect()
}

fn select_target(position: Position, radius: u32, readings: &[PatchReading]) -> Option<Id> {
    readings.iter()
        .filter(|patch| patch.biomass_units > 0
            && distance(position, patch.position) <= u64::from(radius))
        .min_by_key(|patch| (distance(position, patch.position), patch.id))
        .map(|patch| patch.id)
}

fn bite_selected(actor: &mut Actor, patches: &mut [Patch]) -> Result<u32, Rejection> {
    if actor.reserve_units > actor.capacity_units { return Err(Rejection::InvalidReserve); }
    let target = actor.target.ok_or(Rejection::NoTarget)?;
    let patch = patches.iter_mut().find(|patch| patch.id == target)
        .ok_or(Rejection::MissingTarget)?;
    if patch.biomass_units == 0 { return Err(Rejection::EmptyTarget); }
    if actor.position != patch.position { return Err(Rejection::NotInContact); }
    if actor.reserve_units == actor.capacity_units { return Err(Rejection::NoRoom); }
    actor.reserve_units += 1;
    patch.biomass_units -= 1;
    Ok(1)
}

fn main() {
    let position = Position { x: 0, y: 0 };
    let mut patches = [Patch { id: Id(3), position, biomass_units: 1 }];
    let mut actor = Actor { position, reserve_units: 0, capacity_units: 10, target: None };
    let readings = observe_local(position, 2, &patches);
    actor.target = select_target(position, 2, &readings);
    println!("chosen={:?}", actor.target);
    patches[0].biomass_units = 0; // Authored intervening depletion, after observation.
    println!("action={:?}", bite_selected(&mut actor, &mut patches));
    println!("reserve={} biomass={}", actor.reserve_units, patches[0].biomass_units);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn origin() -> Position { Position { x: 0, y: 0 } }
    fn actor() -> Actor {
        Actor { position: origin(), reserve_units: 0, capacity_units: 10, target: None }
    }
    fn reading(id: u32, x: i32, y: i32, biomass_units: u32) -> PatchReading {
        PatchReading { id: Id(id), position: Position { x, y }, biomass_units }
    }

    #[test]
    fn local_observation_excludes_distant_supply() {
        let patches = [
            Patch { id: Id(9), position: Position { x: 2, y: 0 }, biomass_units: 1 },
            Patch { id: Id(1), position: Position { x: 3, y: 0 }, biomass_units: 100 },
        ];
        let readings = observe_local(origin(), 2, &patches);
        assert_eq!(readings.iter().map(|p| p.id).collect::<Vec<_>>(), vec![Id(9)]);
        assert_eq!(select_target(origin(), 2, &readings), Some(Id(9)));
    }

    #[test]
    fn ties_choose_lowest_id_independently_of_input_order() {
        let mut readings = [reading(9, 1, 0, 3), reading(2, 0, 1, 3)];
        assert_eq!(select_target(origin(), 4, &readings), Some(Id(2)));
        readings.reverse();
        assert_eq!(select_target(origin(), 4, &readings), Some(Id(2)));
    }

    #[test]
    fn distance_beats_identity_and_empty_supply_is_ineligible() {
        let readings = [reading(1, 4, 0, 10), reading(9, 2, 0, 1), reading(0, 1, 0, 0)];
        assert_eq!(select_target(origin(), 4, &readings), Some(Id(9)));
    }

    #[test]
    fn empty_or_out_of_range_readings_do_not_invent_a_target() {
        assert_eq!(select_target(origin(), 2, &[]), None);
        assert_eq!(select_target(origin(), 2, &[reading(3, 0, 0, 0)]), None);
        assert_eq!(select_target(origin(), 2, &[reading(3, 3, 0, 10)]), None);
    }

    #[test]
    fn depletion_after_observation_rejects_without_partial_state() {
        let mut actor = actor();
        let mut patches = [Patch { id: Id(3), position: origin(), biomass_units: 1 }];
        let readings = observe_local(actor.position, 2, &patches);
        actor.target = select_target(actor.position, 2, &readings);
        patches[0].biomass_units = 0;
        let before = (actor, patches);
        assert_eq!(bite_selected(&mut actor, &mut patches), Err(Rejection::EmptyTarget));
        assert_eq!((actor, patches), before);
        assert_eq!(readings[0].biomass_units, 1);
    }

    #[test]
    fn removed_target_does_not_redirect_the_action() {
        let mut actor = actor();
        let mut patches = vec![
            Patch { id: Id(3), position: origin(), biomass_units: 1 },
            Patch { id: Id(4), position: origin(), biomass_units: 5 },
        ];
        let readings = observe_local(actor.position, 2, &patches);
        actor.target = select_target(actor.position, 2, &readings);
        patches.retain(|patch| patch.id != Id(3));
        let before_actor = actor;
        let before_patches = patches.clone();
        assert_eq!(bite_selected(&mut actor, &mut patches), Err(Rejection::MissingTarget));
        assert_eq!(actor, before_actor);
        assert_eq!(patches, before_patches);
    }

    #[test]
    fn seeing_a_target_does_not_mean_contact_at_action_time() {
        let mut actor = actor();
        let mut patches = [Patch {
            id: Id(3), position: Position { x: 1, y: 0 }, biomass_units: 3,
        }];
        let readings = observe_local(actor.position, 2, &patches);
        actor.target = select_target(actor.position, 2, &readings);
        let before = (actor, patches);
        assert_eq!(bite_selected(&mut actor, &mut patches), Err(Rejection::NotInContact));
        assert_eq!((actor, patches), before);
    }

    #[test]
    fn successful_bite_checks_room_and_reports_only_actual_gain() {
        let mut actor = Actor { reserve_units: 9, target: Some(Id(3)), ..actor() };
        let mut patches = [Patch { id: Id(3), position: origin(), biomass_units: 2 }];
        assert_eq!(bite_selected(&mut actor, &mut patches), Ok(1));
        assert_eq!((actor.reserve_units, patches[0].biomass_units), (10, 1));
        let before = (actor, patches);
        assert_eq!(bite_selected(&mut actor, &mut patches), Err(Rejection::NoRoom));
        assert_eq!((actor, patches), before);
    }

    #[test]
    fn extreme_coordinates_do_not_overflow_the_distance() {
        let a = Position { x: i32::MIN, y: i32::MIN };
        let b = Position { x: i32::MAX, y: i32::MAX };
        assert_eq!(distance(a, b), 8_589_934_590);
        let readings = [PatchReading { id: Id(3), position: b, biomass_units: 1 }];
        assert_eq!(select_target(a, u32::MAX, &readings), None);
    }
}
