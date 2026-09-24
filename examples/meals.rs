#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Energy { reserve_units: u32, capacity_units: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Patch { biomass_units: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Meal { biomass_taken: u32, energy_gained: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Rejection { InvalidReserve, NotInContact, ZeroRequest, EmptyPatch, NoRoom }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Consumer { id: u32, energy: Energy, in_contact: bool }

fn transfer(
    energy: &mut Energy,
    patch: &mut Patch,
    requested_biomass: u32,
    in_contact: bool,
) -> Result<Meal, Rejection> {
    if energy.reserve_units > energy.capacity_units { return Err(Rejection::InvalidReserve); }
    if !in_contact { return Err(Rejection::NotInContact); }
    if requested_biomass == 0 { return Err(Rejection::ZeroRequest); }
    if patch.biomass_units == 0 { return Err(Rejection::EmptyPatch); }
    let room = energy.capacity_units - energy.reserve_units;
    if room == 0 { return Err(Rejection::NoRoom); }
    let taken = requested_biomass.min(patch.biomass_units).min(room);
    let next_reserve = energy.reserve_units.checked_add(taken)
        .expect("accepted amount is bounded by available capacity");
    let remaining_biomass = patch.biomass_units - taken;
    energy.reserve_units = next_reserve;
    patch.biomass_units = remaining_biomass;
    Ok(Meal { biomass_taken: taken, energy_gained: taken })
}

fn resolve(consumers: &mut [Consumer], patch: &mut Patch, requested_biomass: u32)
    -> Vec<(u32, Result<Meal, Rejection>)>
{
    let mut order: Vec<usize> = (0..consumers.len()).collect();
    order.sort_by_key(|&index| consumers[index].id);
    let mut outcomes = Vec::new();
    for index in order {
        let consumer = &mut consumers[index];
        let outcome = transfer(&mut consumer.energy, patch, requested_biomass, consumer.in_contact);
        outcomes.push((consumer.id, outcome));
    }
    outcomes
}

fn pair() -> [Consumer; 2] {
    [Consumer { id: 9, energy: Energy { reserve_units: 0, capacity_units: 10 }, in_contact: true },
     Consumer { id: 2, energy: Energy { reserve_units: 0, capacity_units: 10 }, in_contact: true }]
}

fn main() {
    let mut consumers = pair();
    let mut patch = Patch { biomass_units: 5 };
    for (id, outcome) in resolve(&mut consumers, &mut patch, 3) {
        println!("id={id} result={outcome:?}");
    }
    println!("remaining biomass={}", patch.biomass_units);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_limits_both_depletion_and_gain() {
        let mut energy = Energy { reserve_units: 8, capacity_units: 10 };
        let mut patch = Patch { biomass_units: 9 };
        assert_eq!(transfer(&mut energy, &mut patch, 4, true),
            Ok(Meal { biomass_taken: 2, energy_gained: 2 }));
        assert_eq!((energy.reserve_units, patch.biomass_units), (10, 7));
    }

    #[test]
    fn lack_of_contact_rejects_without_partial_mutation() {
        let mut energy = Energy { reserve_units: 2, capacity_units: 10 };
        let mut patch = Patch { biomass_units: 5 };
        let before = (energy, patch);
        assert_eq!(transfer(&mut energy, &mut patch, 3, false), Err(Rejection::NotInContact));
        assert_eq!((energy, patch), before);
    }

    #[test]
    fn invalid_reserve_rejects_before_unsigned_subtraction() {
        let mut energy = Energy { reserve_units: 11, capacity_units: 10 };
        let mut patch = Patch { biomass_units: 5 };
        let before = (energy, patch);
        assert_eq!(transfer(&mut energy, &mut patch, 3, true), Err(Rejection::InvalidReserve));
        assert_eq!((energy, patch), before);
    }

    #[test]
    fn zero_request_empty_patch_and_full_reserve_each_leave_state_unchanged() {
        for (reserve, biomass, request, expected) in [
            (2, 5, 0, Rejection::ZeroRequest),
            (2, 0, 3, Rejection::EmptyPatch),
            (10, 5, 3, Rejection::NoRoom),
        ] {
            let mut energy = Energy { reserve_units: reserve, capacity_units: 10 };
            let mut patch = Patch { biomass_units: biomass };
            let before = (energy, patch);
            assert_eq!(transfer(&mut energy, &mut patch, request, true), Err(expected));
            assert_eq!((energy, patch), before);
        }
    }

    #[test]
    fn stable_id_order_resolves_the_same_competition_after_input_reversal() {
        let mut a = pair();
        let mut b = pair();
        b.reverse();
        let mut patch_a = Patch { biomass_units: 5 };
        let mut patch_b = patch_a;
        let outcomes = resolve(&mut a, &mut patch_a, 3);
        assert_eq!(outcomes, vec![
            (2, Ok(Meal { biomass_taken: 3, energy_gained: 3 })),
            (9, Ok(Meal { biomass_taken: 2, energy_gained: 2 })),
        ]);
        assert_eq!(resolve(&mut b, &mut patch_b, 3), outcomes);
        assert_eq!((patch_a.biomass_units, patch_b.biomass_units), (0, 0));
        assert_eq!(a.iter().map(|c| c.energy.reserve_units).sum::<u32>(), 5);
        assert_eq!(a.iter().find(|c| c.id == 2).unwrap().energy.reserve_units, 3);
        assert_eq!(a.iter().find(|c| c.id == 9).unwrap().energy.reserve_units, 2);
    }

    #[test]
    fn depleted_patch_cannot_reward_a_second_resolution() {
        let mut consumers = pair();
        let mut patch = Patch { biomass_units: 5 };
        resolve(&mut consumers, &mut patch, 3);
        let before = consumers;
        assert_eq!(resolve(&mut consumers, &mut patch, 3),
            vec![(2, Err(Rejection::EmptyPatch)), (9, Err(Rejection::EmptyPatch))]);
        assert_eq!(consumers, before);
        assert_eq!(patch.biomass_units, 0);
    }

    #[test]
    fn bounding_before_addition_handles_the_numeric_limit() {
        let mut energy = Energy { reserve_units: u32::MAX - 2, capacity_units: u32::MAX };
        let mut patch = Patch { biomass_units: u32::MAX };
        assert_eq!(transfer(&mut energy, &mut patch, u32::MAX, true),
            Ok(Meal { biomass_taken: 2, energy_gained: 2 }));
        assert_eq!(energy.reserve_units, u32::MAX);
        assert_eq!(patch.biomass_units, u32::MAX - 2);
    }
}
