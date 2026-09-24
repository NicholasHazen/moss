//! Matched deterministic placements; protection is neither food nor a survival guarantee.

use moss_course_ecosystem::{
    Cell, CourseWorld, HuntConfig, MobileScenario, RefugeConfig, RefugeId, RefugeSeed, RestPolicy,
};

fn stores(world: &CourseWorld) -> u64 {
    let s = world.snapshot();
    s.grazers
        .iter()
        .map(|g| u64::from(g.reserve_units))
        .sum::<u64>()
        + s.patches
            .iter()
            .map(|p| u64::from(p.biomass_units))
            .sum::<u64>()
        + world
            .hunt_snapshot()
            .unwrap()
            .hunters
            .iter()
            .map(|h| u64::from(h.reserve_units))
            .sum::<u64>()
}

fn main() {
    let common = MobileScenario::meadow()
        .with_rest(RestPolicy::default())
        .with_hunters(HuntConfig::meadow());
    println!(
        "Matched placements, horizon120, initial stores128 (grazer72 + biomass24 + hunter32). No random draws."
    );
    println!(
        "Every actor, position, trait, supply, birth/rest/hunt rule and history limit is identical: {common:?}"
    );
    println!(
        "Only refuge configuration changes; the selector neither seeks refuge nor flees. Transit protection applies at current cells, not every crossed cell."
    );
    for (label, cells) in [
        ("disabled", None),
        ("food-cell", Some(vec![Cell { x: 3, y: 2 }])),
        ("transit-cell", Some(vec![Cell { x: 4, y: 2 }])),
        (
            "both-food-cells",
            Some(vec![Cell { x: 3, y: 2 }, Cell { x: 7, y: 2 }]),
        ),
    ] {
        let spec = match cells {
            None => common.clone(),
            Some(cells) => common.clone().with_refuges(RefugeConfig::new(
                cells
                    .into_iter()
                    .enumerate()
                    .map(|(i, cell)| RefugeSeed {
                        id: RefugeId(i as u32 + 1),
                        cell,
                    })
                    .collect(),
            )),
        };
        println!("\n{label}: sites={:?}", spec.refuges());
        let mut world = CourseWorld::new_mobile(spec).unwrap();
        assert_eq!(stores(&world), 128);
        let (mut growth, mut upkeep, mut birth_transfer, mut birth_cost) = (0, 0, 0, 0);
        for tick in 1..=120 {
            let before = stores(&world);
            world.step();
            let m = world.mobile_snapshot().unwrap();
            let h = world.hunt_snapshot().unwrap();
            growth += m.base.ledger.added_units;
            upkeep += m.base.ledger.maintenance_units;
            birth_transfer += m.population.ledger.transferred_units;
            birth_cost += m.population.ledger.dissipated_units;
            assert_eq!(
                stores(&world)
                    + m.base.ledger.maintenance_units
                    + m.ledger.travel_units
                    + m.population.ledger.dissipated_units
                    + h.ledger.maintenance_units
                    + h.ledger.travel_units
                    + h.ledger.attack_units,
                before + m.base.ledger.added_units
            );
            assert_eq!(
                m.population.living as u64 + m.population.totals.starvations + h.totals.captures,
                4 + m.population.totals.births
            );
            if [1, 2, 4, 8, 12, 40, 80, 120].contains(&tick) {
                let protected = world
                    .refuge_snapshot()
                    .map(|r| r.grazers.iter().filter(|g| g.refuge.is_some()).count());
                println!(
                    "t{tick} living{} protected{protected:?} hunters{} births{} starvations{} captures{} hunter_starvations{} stores{}",
                    m.population.living,
                    h.living_hunters,
                    m.population.totals.births,
                    m.population.totals.starvations,
                    h.totals.captures,
                    h.totals.hunter_starvations,
                    stores(&world)
                );
            }
        }
        let m = world.mobile_snapshot().unwrap();
        let h = world.hunt_snapshot().unwrap();
        let r = world.rest_snapshot().unwrap();
        println!(
            "actual growth{growth} upkeep{upkeep} grazer_travel{} birth_transfer{birth_transfer} birth_cost{birth_cost} hunter_upkeep{} hunter_travel{} attack{} prey_transfer{} cap_pairs{} cap_ticks{}",
            m.totals.travel_units,
            h.totals.maintenance_units,
            h.totals.travel_units,
            h.totals.attack_units,
            h.totals.transferred_units,
            m.population.totals.capacity_blocked_pairs,
            m.population.totals.capacity_blocked_ticks
        );
        println!(
            "final grazers={:?}; patches={:?}; hunters={:?}; memberships={:?}",
            m.base
                .grazers
                .iter()
                .map(|g| (g.id, g.reserve_units))
                .collect::<Vec<_>>(),
            m.base
                .patches
                .iter()
                .map(|p| (p.id, p.biomass_units))
                .collect::<Vec<_>>(),
            h.hunters
                .iter()
                .map(|h| (h.id, h.reserve_units))
                .collect::<Vec<_>>(),
            world.refuge_snapshot().map(|r| r.grazers)
        );
        println!(
            "rest actions grazer{} hunter{}; complete histories (after,through,lost): base{:?} population{:?} mobile{:?} rest{:?} hunt{:?}; captures1..120={:?}; starvations1..120={:?}",
            r.totals.rest_actions,
            h.totals.rest_actions,
            (
                m.base.history.complete_after_tick,
                m.base.history.collected_through_tick,
                m.base.history.evicted_events
            ),
            (
                m.population.history.complete_after_tick,
                m.population.history.collected_through_tick,
                m.population.history.evicted_events
            ),
            (
                m.history.complete_after_tick,
                m.history.collected_through_tick,
                m.history.evicted_events
            ),
            (
                r.history.complete_after_tick,
                r.history.collected_through_tick,
                r.history.evicted_events
            ),
            (
                h.history.complete_after_tick,
                h.history.collected_through_tick,
                h.history.evicted_events
            ),
            h.history.captures_between(1, 120),
            m.base.history.starvations_between(1, 120)
        );
    }
}
