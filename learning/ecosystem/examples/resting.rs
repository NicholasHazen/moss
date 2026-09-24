use moss_course_ecosystem::{CourseWorld, MobileScenario, RestPolicy};

fn main() {
    for recovery in [2, 6, 10] {
        let mut world =
            CourseWorld::new_mobile(MobileScenario::resting_journey().with_rest(RestPolicy {
                recovery_points_per_tick: recovery,
                ..Default::default()
            }))
            .unwrap();
        println!(
            "journey recovery={recovery}: tick x reserve fatigue activity observed_tick last_rest"
        );
        for _ in 0..10 {
            world.step();
            let m = world.mobile_snapshot().unwrap();
            let r = world.rest_snapshot().unwrap();
            println!(
                "{} {} {} {} {:?} {:?} {:?}",
                r.tick,
                m.grazers[0].cell.x,
                m.base.grazers[0].reserve_units,
                r.grazers[0].fatigue_points,
                r.grazers[0].activity,
                m.grazers[0].observed_tick,
                r.grazers[0].last_rest_tick
            );
        }
    }
    for minimum_rest_ticks in [2, 3] {
        let mut world = CourseWorld::new_mobile(MobileScenario::meadow().with_rest(RestPolicy {
            minimum_rest_ticks,
            ..Default::default()
        }))
        .unwrap();
        println!(
            "meadow minimum={minimum_rest_ticks}, otherwise MobileScenario::meadow + RestPolicy::default, horizon120"
        );
        let mut upkeep = 0;
        let mut growth = 0;
        let mut birth_cost = 0;
        for tick in 1..=120 {
            world.step();
            let m = world.mobile_snapshot().unwrap();
            let r = world.rest_snapshot().unwrap();
            upkeep += m.base.ledger.maintenance_units;
            growth += m.base.ledger.added_units;
            birth_cost += m.population.ledger.dissipated_units;
            if [4, 8, 12, 40, 80, 120].contains(&tick) {
                let reserves: u64 = m
                    .base
                    .grazers
                    .iter()
                    .map(|g| u64::from(g.reserve_units))
                    .sum();
                let food: u64 = m
                    .base
                    .patches
                    .iter()
                    .map(|p| u64::from(p.biomass_units))
                    .sum();
                println!(
                    "tick={tick} living={} juveniles={} births={} starvations={} travel={} reserves={reserves} biomass={food} rest={:?}",
                    m.base.grazers.len(),
                    m.population.juveniles,
                    m.population.totals.births,
                    m.population.totals.starvations,
                    m.totals.travel_cells,
                    r.totals
                );
            }
        }
        let m = world.mobile_snapshot().unwrap();
        let r = world.rest_snapshot().unwrap();
        println!(
            "growth={growth} upkeep={upkeep} birth_cost={birth_cost} final_ids={:?}",
            m.base.grazers.iter().map(|g| g.id).collect::<Vec<_>>()
        );
        println!(
            "coverage base({},{}] lost{} population({},{}] lost{} mobile({},{}] lost{} rest({},{}] lost{}",
            m.base.history.complete_after_tick,
            m.tick,
            m.base.history.evicted_events,
            m.population.history.complete_after_tick,
            m.tick,
            m.population.history.evicted_events,
            m.history.complete_after_tick,
            m.tick,
            m.history.evicted_events,
            r.history.complete_after_tick,
            r.tick,
            r.history.evicted_events
        );
    }
}
