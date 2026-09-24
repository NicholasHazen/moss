#[derive(Debug, PartialEq, Eq)]
struct Reading {
    actor: u64,
    name: String,
    reserve_units: Option<u32>,
}

fn same_value<T: PartialEq>(actual: &T, expected: &T) -> bool {
    actual == expected
}

macro_rules! assert_reading {
    ($reading:expr, actor = $actor:expr, reserve = $reserve:expr $(,)?) => {{
        let reading = &$reading;
        let actor: u64 = $actor;
        let reserve: Option<u32> = $reserve;
        assert_eq!(reading.actor, actor, "actor identity");
        assert_eq!(reading.reserve_units, reserve, "reserve measurement");
    }};
}

fn fixture() -> Reading {
    Reading {
        actor: 7,
        name: "Fern".into(),
        reserve_units: Some(0),
    }
}

fn main() {
    let reading = fixture();
    let mut evaluations = 0;
    assert_reading!(
        {
            evaluations += 1;
            &reading
        },
        actor = 7,
        reserve = Some(0)
    );
    println!(
        "{}: actor={} reserve={:?}",
        reading.name, reading.actor, reading.reserve_units
    );
    println!("source evaluations: {evaluations}");
    println!("zero equals unknown: {}", same_value(&Some(0_u32), &None));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::catch_unwind;

    #[test]
    fn accepts_values_references_and_an_optional_trailing_comma() {
        let reading = fixture();
        assert_reading!(reading, actor = 7, reserve = Some(0));
        assert_reading!(&reading, actor = 7, reserve = Some(0),);
        assert_reading!(fixture(), actor = 7, reserve = Some(0));
    }

    #[test]
    fn borrowing_the_reading_does_not_move_its_owned_name() {
        let reading = fixture();
        assert_reading!(reading, actor = 7, reserve = Some(0));
        assert_eq!(reading.name, "Fern");
    }

    #[test]
    fn each_input_runs_once_in_the_documented_order() {
        let mut calls: Vec<&str> = Vec::new();
        assert_reading!(
            {
                calls.push("reading");
                fixture()
            },
            actor = {
                calls.push("actor");
                7
            },
            reserve = {
                calls.push("reserve");
                Some(0)
            },
        );
        assert_eq!(calls, vec!["reading", "actor", "reserve"]);
    }

    #[test]
    fn generated_bindings_do_not_capture_callers_with_the_same_names() {
        let reading = fixture();
        let actor = 7_u64;
        let reserve = Some(0_u32);
        assert_reading!(reading, actor = actor, reserve = reserve);
        assert_eq!((actor, reserve), (7, Some(0)));
        assert_eq!(reading.name, "Fern");
    }

    #[test]
    fn measured_zero_does_not_match_unknown() {
        let result = catch_unwind(|| {
            assert_reading!(fixture(), actor = 7, reserve = None);
        });
        let panic = result.expect_err("wrong reserve must fail");
        let message = panic.downcast_ref::<String>().unwrap();
        assert!(message.contains("reserve measurement"), "{message}");
    }

    #[test]
    fn wrong_identity_is_rejected_even_with_a_matching_reserve() {
        let result = catch_unwind(|| {
            assert_reading!(fixture(), actor = 8, reserve = Some(0));
        });
        let panic = result.expect_err("wrong actor must fail");
        let message = panic.downcast_ref::<String>().unwrap();
        assert!(message.contains("actor identity"), "{message}");
    }

    #[test]
    fn unknown_can_be_the_expected_measurement() {
        let reading = Reading {
            reserve_units: None,
            ..fixture()
        };
        assert_reading!(reading, actor = 7, reserve = None);
        assert!(same_value(&reading.reserve_units, &None));
    }

    #[test]
    fn generic_function_handles_other_types_without_new_syntax() {
        assert!(same_value(&String::from("Fern"), &String::from("Fern")));
        assert!(!same_value(&vec![2, 10], &vec![10, 2]));
        assert!(same_value(&None::<u32>, &None));
    }
}
