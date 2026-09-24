#[derive(Clone, Debug, PartialEq, Eq)]
struct Record {
    tick: u64,
    actor: u64,
    kind: String,
    note: String,
    reserve_units: Option<u32>,
}

#[derive(Debug, Default)]
struct Journal {
    records: Vec<Record>,
}

#[derive(Debug, PartialEq, Eq)]
struct View<'a> {
    tick: u64,
    actor: u64,
    note: &'a str,
    reserve_units: Option<u32>,
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    tick: u64,
    actor: u64,
    note: String,
    reserve_units: Option<u32>,
}

impl View<'_> {
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            tick: self.tick,
            actor: self.actor,
            note: self.note.to_owned(),
            reserve_units: self.reserve_units,
        }
    }
}

fn latest_with_kind<'a>(journal: &'a Journal, actor: u64, kind: &str) -> Option<View<'a>> {
    journal
        .records
        .iter()
        .rev()
        .find(|record| record.actor == actor && record.kind == kind)
        .map(|record| View {
            tick: record.tick,
            actor: record.actor,
            note: &record.note,
            reserve_units: record.reserve_units,
        })
}

fn last_note(journal: &Journal) -> Option<&str> {
    journal.records.last().map(|record| record.note.as_str())
}

fn fixture() -> Journal {
    Journal {
        records: vec![
            Record {
                tick: 2,
                actor: 7,
                kind: "meal".into(),
                note: "ate 2 units".into(),
                reserve_units: Some(9),
            },
            Record {
                tick: 4,
                actor: 7,
                kind: "observation".into(),
                note: "reserve uncollected".into(),
                reserve_units: None,
            },
            Record {
                tick: 5,
                actor: 7,
                kind: "meal".into(),
                note: "ate 1 unit".into(),
                reserve_units: Some(10),
            },
            Record {
                tick: 6,
                actor: 8,
                kind: "meal".into(),
                note: "ate 3 units".into(),
                reserve_units: Some(12),
            },
        ],
    }
}

fn main() {
    let mut journal = fixture();
    let view = {
        let search = String::from("meal");
        latest_with_kind(&journal, 7, &search).unwrap()
    };
    println!(
        "view: actor={} tick={} note={}",
        view.actor, view.tick, view.note
    );
    let snapshot = view.snapshot();
    journal.records.clear();
    println!("retained rows: {}", journal.records.len());
    println!(
        "export: actor={} tick={} note={} reserve={:?}",
        snapshot.actor, snapshot.tick, snapshot.note, snapshot.reserve_units
    );
    assert_eq!(last_note(&journal), None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_latest_matching_actor_and_kind_in_append_order() {
        let journal = fixture();
        let view = latest_with_kind(&journal, 7, "meal").unwrap();
        assert_eq!((view.tick, view.actor, view.note), (5, 7, "ate 1 unit"));
        assert_eq!(view.reserve_units, Some(10));
    }

    #[test]
    fn append_order_is_not_replaced_by_a_greatest_tick_search() {
        let mut journal = fixture();
        journal.records.push(Record {
            tick: 1,
            actor: 7,
            kind: "meal".into(),
            note: "late archival copy".into(),
            reserve_units: None,
        });
        let view = latest_with_kind(&journal, 7, "meal").unwrap();
        assert_eq!((view.tick, view.note), (1, "late archival copy"));
    }

    #[test]
    fn empty_or_absent_match_is_none() {
        assert_eq!(latest_with_kind(&Journal::default(), 7, "meal"), None);
        let journal = fixture();
        assert_eq!(latest_with_kind(&journal, 99, "meal"), None);
        assert_eq!(latest_with_kind(&journal, 7, "birth"), None);
    }

    #[test]
    fn view_borrows_the_original_note_allocation() {
        let journal = fixture();
        let view = latest_with_kind(&journal, 7, "meal").unwrap();
        assert!(std::ptr::eq(view.note, journal.records[2].note.as_str()));
    }

    #[test]
    fn query_need_not_live_as_long_as_the_returned_view() {
        let journal = fixture();
        let view = {
            let query = String::from("meal");
            latest_with_kind(&journal, 7, &query).unwrap()
        };
        assert_eq!(view.note, "ate 1 unit");
    }

    #[test]
    fn snapshot_survives_mutation_eviction_and_owner_drop() {
        let snapshot = {
            let mut journal = fixture();
            let snapshot = latest_with_kind(&journal, 7, "meal").unwrap().snapshot();
            journal.records[2].note.push_str("; amended");
            journal.records.clear();
            snapshot
        };
        assert_eq!(snapshot.note, "ate 1 unit");
        assert_eq!((snapshot.tick, snapshot.actor), (5, 7));
    }

    #[test]
    fn uncollected_reserve_and_measured_zero_stay_distinct() {
        let mut journal = fixture();
        let unknown = latest_with_kind(&journal, 7, "observation")
            .unwrap()
            .snapshot();
        journal.records[1].reserve_units = Some(0);
        let zero = latest_with_kind(&journal, 7, "observation")
            .unwrap()
            .snapshot();
        assert_eq!(unknown.reserve_units, None);
        assert_eq!(zero.reserve_units, Some(0));
        assert_ne!(unknown, zero);
    }

    #[test]
    fn unicode_note_is_borrowed_whole_and_single_input_elision_works() {
        let mut journal = Journal::default();
        journal.records.push(Record {
            tick: 1,
            actor: 7,
            kind: "observation".into(),
            note: "Fern — 水".into(),
            reserve_units: None,
        });
        assert_eq!(last_note(&journal), Some("Fern — 水"));
        assert_eq!(
            latest_with_kind(&journal, 7, "observation").unwrap().note,
            "Fern — 水"
        );
    }
}
