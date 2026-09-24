use moss_course_ecosystem::{CourseWorld, HuntConfig, MobileScenario, RestPolicy};
fn main() {
    let mut contest = CourseWorld::new_mobile(MobileScenario::hunting_contention()).unwrap();
    for _ in 0..3 {
        contest.step();
        let m = contest.mobile_snapshot().unwrap();
        let h = contest.hunt_snapshot().unwrap();
        println!(
            "contention t{} grazers{:?} patch{} hunters{:?} counts{:?}",
            m.tick,
            m.base
                .grazers
                .iter()
                .map(|g| (g.id, g.reserve_units))
                .collect::<Vec<_>>(),
            m.base.patches[0].biomass_units,
            h.hunters
                .iter()
                .map(|h| (
                    h.id,
                    h.reserve_units,
                    h.target.map(|t| t.prey),
                    h.fatigue_points
                ))
                .collect::<Vec<_>>(),
            h.totals
        );
    }
    for minimum_rest_ticks in [2, 3] {
        for hunters in [false, true] {
            let mut spec = MobileScenario::meadow().with_rest(RestPolicy {
                minimum_rest_ticks,
                ..Default::default()
            });
            if hunters {
                spec = spec.with_hunters(HuntConfig::meadow());
            }
            let mut world = CourseWorld::new_mobile(spec).unwrap();
            let mut growth = 0;
            let mut upkeep = 0;
            let mut birth_cost = 0;
            println!(
                "pilot min={minimum_rest_ticks} hunters={hunters} initial stores={} horizon120",
                if hunters { 128 } else { 96 }
            );
            for tick in 1..=120 {
                world.step();
                let m = world.mobile_snapshot().unwrap();
                let r = world.rest_snapshot().unwrap();
                let h = world.hunt_snapshot();
                growth += m.base.ledger.added_units;
                upkeep += m.base.ledger.maintenance_units;
                birth_cost += m.population.ledger.dissipated_units;
                if [1, 2, 4, 8, 12, 40, 80, 120].contains(&tick) {
                    let reserves: u64 = m
                        .base
                        .grazers
                        .iter()
                        .map(|g| u64::from(g.reserve_units))
                        .sum();
                    let biomass: u64 = m
                        .base
                        .patches
                        .iter()
                        .map(|p| u64::from(p.biomass_units))
                        .sum();
                    let hunter_reserves: u64 = h.as_ref().map_or(0, |h| {
                        h.hunters.iter().map(|g| u64::from(g.reserve_units)).sum()
                    });
                    println!(
                        "t{tick} grazers{} juveniles{} births{} starved{} travel{} grest{} gstores{reserves} biomass{biomass} hstores{hunter_reserves} hunters{} hcounts{:?}",
                        m.base.grazers.len(),
                        m.population.juveniles,
                        m.population.totals.births,
                        m.population.totals.starvations,
                        m.totals.travel_cells,
                        r.totals.rest_actions,
                        h.as_ref().map_or(0, |h| h.living_hunters),
                        h.as_ref().map(|h| h.totals)
                    );
                }
            }
            let m = world.mobile_snapshot().unwrap();
            let r = world.rest_snapshot().unwrap();
            println!(
                "growth{growth} grazerupkeep{upkeep} birthcost{birth_cost} finalgrazers{:?} grest{:?}",
                m.base.grazers.iter().map(|g| g.id).collect::<Vec<_>>(),
                r.totals
            );
            println!(
                "coverage base({},120]lost{} pop({},120]lost{} mobile({},120]lost{} rest({},120]lost{} hunt{:?}",
                m.base.history.complete_after_tick,
                m.base.history.evicted_events,
                m.population.history.complete_after_tick,
                m.population.history.evicted_events,
                m.history.complete_after_tick,
                m.history.evicted_events,
                r.history.complete_after_tick,
                r.history.evicted_events,
                world
                    .hunt_snapshot()
                    .map(|h| (h.history.complete_after_tick, h.history.evicted_events))
            );
        }
    }
}
