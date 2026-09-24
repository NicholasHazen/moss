#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Reserve { units: u32 }

#[derive(Debug, PartialEq, Eq)]
struct Observation {
    name: String,
    reserve: Reserve,
}

fn capture(name: &str, reserve: &Reserve) -> Observation {
    Observation { name: name.to_owned(), reserve: *reserve }
}

fn main() {
    let nickname = String::from("Fern");
    let mut reserve = Reserve { units: 60 };
    let reading = capture(&nickname, &reserve);
    reserve.units = 57;
    let saved = reading;
    println!("{}: live={} saved={}", nickname, reserve.units, saved.reserve.units);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_preserves_the_source() {
        let name = String::from("Fern");
        let reserve = Reserve { units: 60 };
        let reading = capture(&name, &reserve);
        assert_eq!(name, "Fern");
        assert_eq!(reserve.units, 60);
        assert_eq!(reading.name, "Fern");
        assert_eq!(reading.reserve.units, 60);
    }

    #[test]
    fn captured_reserve_is_independent() {
        let mut reserve = Reserve { units: 60 };
        let reading = capture("Fern", &reserve);
        reserve.units = 57;
        assert_eq!(reserve.units, 57);
        assert_eq!(reading.reserve.units, 60);
    }

    #[test]
    fn captured_text_is_independent() {
        let mut name = String::from("Fern");
        let reading = capture(&name, &Reserve { units: 60 });
        name.push_str(" the hare");
        assert_eq!(name, "Fern the hare");
        assert_eq!(reading.name, "Fern");
    }
}
