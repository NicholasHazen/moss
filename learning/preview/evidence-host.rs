// Appended to the exact checked lesson program. No projection arithmetic is
// repeated here: the host only supplies a fixture and reads its resources.
use std::cell::RefCell;

struct BrowserEvidence {
    world: World,
    schedule: Schedule,
}

impl BrowserEvidence {
    fn new(used: u32, capacity: u32, selected: u32, ordering: u32) -> Self {
        let mut world = fixture();
        let selected_entities: Vec<_> = world
            .query_filtered::<Entity, With<Selected>>()
            .iter(&world)
            .collect();
        for entity in selected_entities {
            world.entity_mut(entity).insert(Sample { used, capacity });
            if selected == 0 {
                world.entity_mut(entity).remove::<Selected>();
            }
        }
        if selected > 1 {
            world.spawn((Selected, Sample { used: 1, capacity: 2 }));
        }
        let schedule = match ordering {
            1 => {
                let mut schedule = Schedule::default();
                schedule.set_executor_kind(ExecutorKind::SingleThreaded);
                schedule.add_systems(observe_display);
                schedule
            }
            2 => {
                let mut schedule = Schedule::default();
                schedule.set_executor_kind(ExecutorKind::SingleThreaded);
                schedule.add_systems((observe_display, project_selected).chain());
                schedule
            }
            _ => installed_schedule(),
        };
        Self { world, schedule }
    }
}

thread_local! {
    static BROWSER_EVIDENCE: RefCell<BrowserEvidence> =
        RefCell::new(BrowserEvidence::new(3, 8, 1, 0));
}

// SAFETY: These unique exported names are the only C ABI for this isolated
// single-threaded host. They accept scalar values and expose no raw pointers.
#[unsafe(no_mangle)]
pub extern "C" fn fieldnotes_evidence_reset(used: u32, capacity: u32, selected: u32, ordering: u32) {
    BROWSER_EVIDENCE.with(|state| {
        *state.borrow_mut() = BrowserEvidence::new(used, capacity, selected, ordering);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn fieldnotes_evidence_step() {
    BROWSER_EVIDENCE.with(|state| {
        let BrowserEvidence { world, schedule } = &mut *state.borrow_mut();
        schedule.run(world);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn fieldnotes_evidence_value(field: u32) -> i64 {
    BROWSER_EVIDENCE.with(|state| {
        let state = state.borrow();
        match field {
            0 => state.world.resource::<DisplayValue>().0.map(i64::from).unwrap_or(-1),
            1 => state.world.resource::<Trace>().0.last().copied().flatten().map(i64::from).unwrap_or(-1),
            2 => state.world.resource::<Trace>().0.len() as i64,
            _ => -1,
        }
    })
}
