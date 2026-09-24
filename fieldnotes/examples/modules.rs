mod measurement {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Energy { units: u32 }

    #[derive(Debug, PartialEq, Eq)]
    pub struct InvalidReserve;

    fn check_limit(units: u32) -> Result<(), InvalidReserve> {
        if units <= 100 { Ok(()) } else { Err(InvalidReserve) }
    }

    impl Energy {
        pub fn new(units: u32) -> Result<Self, InvalidReserve> {
            check_limit(units)?;
            Ok(Self { units })
        }
        pub fn units(&self) -> u32 { self.units }
    }
}

pub use measurement::{Energy, InvalidReserve};

mod inspector {
    use crate::Energy;

    pub fn report(name: &str, reserve: Option<&Energy>) -> String {
        match reserve {
            Some(value) => format!("{name}: {} units", value.units()),
            None => format!("{name}: unknown reserve"),
        }
    }
}

pub use inspector::report;

fn main() {
    let reserve = Energy::new(60).expect("the authored fixture is valid");
    println!("{}", report("Fern", Some(&reserve)));
    println!("{}", report("Unmeasured", None));
}

#[cfg(test)]
mod tests {
    use super::{Energy, InvalidReserve, report};

    #[test]
    fn callers_keep_the_same_public_names() {
        let reserve = Energy::new(60).unwrap();
        assert_eq!(report("Fern", Some(&reserve)), "Fern: 60 units");
        assert_eq!(reserve.units(), 60);
    }
    #[test]
    fn missing_measurement_is_not_a_zero_measurement() {
        let zero = Energy::new(0).unwrap();
        assert_eq!(report("Fern", Some(&zero)), "Fern: 0 units");
        assert_eq!(report("Fern", None), "Fern: unknown reserve");
    }
    #[test]
    fn the_constructor_retains_its_boundary() {
        assert_eq!(Energy::new(100).unwrap().units(), 100);
        assert_eq!(Energy::new(101), Err(InvalidReserve));
    }
    #[test]
    fn returned_text_does_not_borrow_the_nickname() {
        let mut nickname = String::from("Fern");
        let text = report(&nickname, None);
        nickname.clear();
        assert_eq!(text, "Fern: unknown reserve");
        assert!(nickname.is_empty());
    }
}
