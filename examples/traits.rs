mod identity {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct StableId(u64);

    #[derive(Debug, PartialEq, Eq)]
    pub struct InvalidId;

    impl StableId {
        pub fn new(value: u64) -> Result<Self, InvalidId> {
            if value == 0 { Err(InvalidId) } else { Ok(Self(value)) }
        }
        pub fn value(self) -> u64 { self.0 }
    }
}

pub use identity::{InvalidId, StableId};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Reading { id: StableId, name: String, reserve: Option<u32> }

fn ordered_unique<T: Ord + Copy>(values: &[T]) -> Vec<T> {
    let mut result = values.to_vec();
    result.sort();
    result.dedup();
    result
}

fn ids_of(readings: &[Reading]) -> Vec<StableId> {
    let ids: Vec<_> = readings.iter().map(|reading| reading.id).collect();
    ordered_unique(&ids)
}

fn main() {
    let fern = Reading {
        id: StableId::new(10).unwrap(), name: "Fern".into(), reserve: Some(57),
    };
    let mut renamed = fern.clone();
    renamed.name.push_str(" the hare");
    println!("same ID: {}", fern.id == renamed.id);
    println!("same reading: {}", fern == renamed);
    let ids = ordered_unique(&[fern.id, StableId::new(2).unwrap(), fern.id]);
    println!("IDs: {:?}", ids.iter().map(|id| id.value()).collect::<Vec<_>>());
    assert_eq!(ids_of(&[fern, renamed]).len(), 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_order_is_numeric_and_duplicates_collapse() {
        let two = StableId::new(2).unwrap();
        let ten = StableId::new(10).unwrap();
        assert_eq!(ordered_unique(&[ten, two, ten]), vec![two, ten]);
        assert_ne!(two, ten);
    }
    #[test]
    fn empty_input_stays_empty() {
        assert_eq!(ordered_unique::<StableId>(&[]), vec![]);
    }
    #[test]
    fn collecting_ids_does_not_take_the_readings() {
        let readings = vec![Reading {
            id: StableId::new(4).unwrap(), name: "Fern".into(), reserve: None,
        }];
        let ids = ids_of(&readings);
        assert_eq!(readings[0].name, "Fern");
        drop(readings);
        assert_eq!(ids[0].value(), 4);
    }
    #[test]
    fn equal_identity_does_not_mean_equal_observation() {
        let first = Reading {
            id: StableId::new(4).unwrap(), name: "Fern".into(), reserve: None,
        };
        let mut later = first.clone();
        later.reserve = Some(0);
        assert_eq!(first.id, later.id);
        assert_ne!(first, later);
    }
    #[test]
    fn cloning_owned_text_produces_independent_text() {
        let first = Reading {
            id: StableId::new(4).unwrap(), name: "Fern".into(), reserve: Some(57),
        };
        let mut second = first.clone();
        second.name.push_str(" the hare");
        assert_eq!(first.name, "Fern");
        assert_eq!(second.name, "Fern the hare");
    }
    #[test]
    fn derives_do_not_replace_constructor_validation() {
        assert_eq!(StableId::new(0), Err(InvalidId));
        let id = StableId::new(1).unwrap();
        let copied = id;
        assert_eq!(id, copied);
    }
    #[test]
    fn the_same_bound_supports_another_ordered_copy_type() {
        assert_eq!(ordered_unique(&[7_i32, -1, 7, 0]), vec![-1, 0, 7]);
    }
}
