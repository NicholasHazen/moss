#[derive(Clone, Debug, PartialEq, Eq)]
struct Report {
    actor: u64,
    tick: u64,
    reserve_units: Option<u32>,
}

trait ReportFormat {
    fn render(&self, report: &Report) -> String;
}

#[derive(Clone)]
enum Style {
    Named { prefix: String },
    Compact,
}

fn reserve_text(report: &Report) -> String {
    match report.reserve_units {
        Some(value) => value.to_string(),
        None => "unknown".into(),
    }
}

fn named_line(prefix: &str, report: &Report) -> String {
    format!(
        "{prefix}: actor={} tick={} reserve={}",
        report.actor,
        report.tick,
        reserve_text(report)
    )
}

fn compact_line(report: &Report) -> String {
    format!("{}@{}:{}", report.actor, report.tick, reserve_text(report))
}

impl ReportFormat for Style {
    fn render(&self, report: &Report) -> String {
        match self {
            Self::Named { prefix } => named_line(prefix, report),
            Self::Compact => compact_line(report),
        }
    }
}

struct NamedFormat {
    prefix: String,
}

struct CompactFormat;

impl ReportFormat for NamedFormat {
    fn render(&self, report: &Report) -> String {
        named_line(&self.prefix, report)
    }
}

impl ReportFormat for CompactFormat {
    fn render(&self, report: &Report) -> String {
        compact_line(report)
    }
}

fn boxed_format(style: Style) -> Box<dyn ReportFormat> {
    match style {
        Style::Named { prefix } => Box::new(NamedFormat { prefix }),
        Style::Compact => Box::new(CompactFormat),
    }
}

fn render_generic<F: ReportFormat>(formatter: &F, report: &Report) -> String {
    formatter.render(report)
}

fn render_borrowed(formatter: &dyn ReportFormat, report: &Report) -> String {
    formatter.render(report)
}

fn render_all(formatters: &[Box<dyn ReportFormat>], report: &Report) -> Vec<String> {
    formatters
        .iter()
        .map(|formatter| formatter.render(report))
        .collect()
}

fn fixture() -> Report {
    Report {
        actor: 7,
        tick: 5,
        reserve_units: Some(10),
    }
}

fn main() {
    let report = fixture();
    let styles = [
        Style::Named {
            prefix: "Inspector".into(),
        },
        Style::Compact,
    ];
    let expected: Vec<_> = styles
        .iter()
        .map(|style| render_generic(style, &report))
        .collect();
    let formatters: Vec<_> = styles.into_iter().map(boxed_format).collect();
    let actual = render_all(&formatters, &report);
    assert_eq!(expected, actual);
    for line in actual {
        println!("{line}");
    }
    println!("borrowed: {}", render_borrowed(&CompactFormat, &report));
    let shared = std::rc::Rc::new(report);
    let second_panel = std::rc::Rc::clone(&shared);
    println!(
        "same allocation: {}",
        std::rc::Rc::ptr_eq(&shared, &second_panel)
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Arc;

    #[test]
    fn enum_and_boxed_adapters_agree_for_each_style_and_absence_case() {
        for reserve_units in [None, Some(0), Some(10)] {
            let report = Report {
                reserve_units,
                ..fixture()
            };
            for style in [
                Style::Named {
                    prefix: "Inspector".into(),
                },
                Style::Compact,
            ] {
                let expected = style.render(&report);
                assert_eq!(boxed_format(style).render(&report), expected);
            }
        }
    }

    #[test]
    fn output_preserves_identity_time_zero_and_unknown() {
        let unknown = Report {
            reserve_units: None,
            ..fixture()
        };
        let zero = Report {
            reserve_units: Some(0),
            ..fixture()
        };
        assert_eq!(boxed_format(Style::Compact).render(&unknown), "7@5:unknown");
        assert_eq!(boxed_format(Style::Compact).render(&zero), "7@5:0");
        assert_eq!(
            boxed_format(Style::Named {
                prefix: "观察".into()
            })
            .render(&fixture()),
            "观察: actor=7 tick=5 reserve=10"
        );
    }

    #[test]
    fn stored_formatter_owns_its_prefix_after_construction_scope_ends() {
        let formatter = {
            let prefix = String::from("Saved");
            boxed_format(Style::Named { prefix })
        };
        assert_eq!(
            formatter.render(&fixture()),
            "Saved: actor=7 tick=5 reserve=10"
        );
    }

    #[test]
    fn heterogeneous_collection_accepts_a_new_formatter_without_an_enum_variant() {
        struct IdOnly;
        impl ReportFormat for IdOnly {
            fn render(&self, report: &Report) -> String {
                format!("ID {}", report.actor)
            }
        }
        let formatters: Vec<Box<dyn ReportFormat>> = vec![
            boxed_format(Style::Compact),
            Box::new(IdOnly),
            boxed_format(Style::Named {
                prefix: "Last".into(),
            }),
        ];
        assert_eq!(
            render_all(&formatters, &fixture()),
            vec!["7@5:10", "ID 7", "Last: actor=7 tick=5 reserve=10"]
        );
        assert!(render_all(&[], &fixture()).is_empty());
    }

    #[test]
    fn borrowed_dispatch_needs_no_box_and_keeps_source_unchanged() {
        let report = fixture();
        let before = report.clone();
        let formatter = Style::Compact;
        assert_eq!(
            render_generic(&formatter, &report),
            render_borrowed(&formatter, &report)
        );
        assert_eq!(report, before);
    }

    #[test]
    fn rc_handles_share_ownership_while_a_report_clone_is_independent() {
        let first = Rc::new(fixture());
        let second = Rc::clone(&first);
        assert!(Rc::ptr_eq(&first, &second));
        let mut independent = (*first).clone();
        independent.reserve_units = Some(0);
        drop(first);
        assert_eq!(second.reserve_units, Some(10));
        assert_eq!(independent.reserve_units, Some(0));
    }

    #[test]
    fn refcell_rejects_overlapping_mutation_until_read_guard_is_dropped() {
        let cache = RefCell::new(String::from("old preview"));
        let reading = cache.borrow();
        assert!(cache.try_borrow_mut().is_err());
        assert_eq!(&*reading, "old preview");
        drop(reading);
        *cache.try_borrow_mut().unwrap() = String::from("new preview");
        assert_eq!(&*cache.borrow(), "new preview");
    }

    #[test]
    fn this_owned_report_can_be_shared_with_a_native_worker() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Report>();
        assert_send_sync::<Arc<Report>>();
        let report = Arc::new(fixture());
        let worker_report = Arc::clone(&report);
        let worker = std::thread::spawn(move || CompactFormat.render(&worker_report));
        assert_eq!(worker.join().unwrap(), "7@5:10");
        assert_eq!(report.reserve_units, Some(10));
    }
}
