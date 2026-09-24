use moss_course_ecosystem::{CourseWorld, Life, Scenario};

fn main() {
    for (label, scenario) in [
        ("limited", Scenario::limited_supply()),
        ("generous", Scenario::generous_supply()),
    ] {
        let mut course = CourseWorld::new(scenario).expect("authored comparison is valid");
        println!("{label}: tick, alive, reserves by ID, biomass");
        for _ in 0..12 {
            course.step();
            let snapshot = course.snapshot();
            let alive = snapshot
                .grazers
                .iter()
                .filter(|grazer| grazer.life == Life::Alive)
                .count();
            let reserves: Vec<_> = snapshot
                .grazers
                .iter()
                .map(|grazer| (grazer.id.0, grazer.reserve_units))
                .collect();
            println!(
                "{}, {alive}, {reserves:?}, {}",
                snapshot.tick, snapshot.patches[0].biomass_units
            );
        }
    }
}
