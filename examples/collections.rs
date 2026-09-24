#[derive(Debug)]
struct Source {
    id: u64,
    name: String,
    reserve: Option<u32>,
    creature: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct Row { id: u64, name: String, reserve: Option<u32> }

#[derive(Debug, PartialEq, Eq)]
enum ReportError { DuplicateId(u64) }

fn collect_report(sources: &[Source]) -> Result<Vec<Row>, ReportError> {
    let mut rows: Vec<Row> = sources.iter()
        .filter(|source| source.creature)
        .map(|source| Row {
            id: source.id,
            name: source.name.clone(),
            reserve: source.reserve,
        })
        .collect();
    rows.sort_by_key(|row| row.id);
    for pair in rows.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(ReportError::DuplicateId(pair[0].id));
        }
    }
    Ok(rows)
}

fn find_row(rows: &[Row], id: u64) -> Option<&Row> {
    rows.iter().find(|row| row.id == id)
}

fn fixture() -> Vec<Source> {
    vec![
        Source { id: 9, name: "Flint".into(), reserve: Some(0), creature: true },
        Source { id: 3, name: "Fern".into(), reserve: Some(57), creature: true },
        Source { id: 7, name: "Meadow".into(), reserve: None, creature: false },
        Source { id: 5, name: "Unmeasured".into(), reserve: None, creature: true },
    ]
}

fn main() {
    let rows = collect_report(&fixture()).expect("fixture IDs are unique");
    for row in &rows {
        println!("#{} {}: {:?}", row.id, row.name, row.reserve);
    }
    assert!(find_row(&rows, 404).is_none());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_a_valid_empty_report() {
        assert_eq!(collect_report(&[]), Ok(vec![]));
    }
    #[test]
    fn reordered_input_has_one_canonical_report() {
        let mut sources = fixture();
        let first = collect_report(&sources).unwrap();
        sources.reverse();
        assert_eq!(collect_report(&sources).unwrap(), first);
        assert_eq!(first.iter().map(|row| row.id).collect::<Vec<_>>(), vec![3, 5, 9]);
    }
    #[test]
    fn unknown_zero_and_unselected_are_different() {
        let rows = collect_report(&fixture()).unwrap();
        assert_eq!(find_row(&rows, 5).unwrap().reserve, None);
        assert_eq!(find_row(&rows, 9).unwrap().reserve, Some(0));
        assert!(find_row(&rows, 7).is_none());
        assert!(find_row(&rows, 404).is_none());
    }
    #[test]
    fn saved_rows_own_their_text_and_measurements() {
        let mut sources = fixture();
        let rows = collect_report(&sources).unwrap();
        sources[0].name.clear();
        sources[0].reserve = Some(80);
        drop(sources);
        let saved = find_row(&rows, 9).unwrap();
        assert_eq!(saved.name, "Flint");
        assert_eq!(saved.reserve, Some(0));
    }
    #[test]
    fn duplicate_identity_is_rejected_instead_of_overwritten() {
        let mut sources = fixture();
        sources[0].id = 3;
        assert_eq!(collect_report(&sources), Err(ReportError::DuplicateId(3)));
    }
    #[test]
    fn lookup_borrows_a_row_already_in_the_report() {
        let rows = collect_report(&fixture()).unwrap();
        assert!(std::ptr::eq(find_row(&rows, 3).unwrap(), &rows[0]));
    }
}
