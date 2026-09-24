#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Units(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Ticks(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UnitsPerTick(u32);

#[derive(Debug, PartialEq, Eq)]
struct Forecast {
    requested: Units,
    paid: Units,
    remaining: Units,
}

fn forecast(start: Units, ticks: Ticks, rate: UnitsPerTick) -> Option<Forecast> {
    let requested = match ticks.0.checked_mul(rate.0) {
        Some(units) => units,
        None => return None,
    };
    let remaining = start.0.saturating_sub(requested);
    Some(Forecast {
        requested: Units(requested),
        paid: Units(start.0 - remaining),
        remaining: Units(remaining),
    })
}

fn main() {
    let report = forecast(Units(4), Ticks(3), UnitsPerTick(2))
        .expect("the example's product fits in u32");
    println!("requested={} paid={} remaining={}",
        report.requested.0, report.paid.0, report.remaining.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_and_rate_both_affect_the_request() {
        assert_eq!(forecast(Units(60), Ticks(3), UnitsPerTick(2)), Some(Forecast {
            requested: Units(6), paid: Units(6), remaining: Units(54),
        }));
    }

    #[test]
    fn actual_payment_cannot_exceed_available_units() {
        assert_eq!(forecast(Units(4), Ticks(3), UnitsPerTick(2)), Some(Forecast {
            requested: Units(6), paid: Units(4), remaining: Units(0),
        }));
    }

    #[test]
    fn zero_duration_and_zero_rate_keep_the_starting_amount() {
        for (ticks, rate) in [(Ticks(0), UnitsPerTick(u32::MAX)),
                              (Ticks(u32::MAX), UnitsPerTick(0))] {
            assert_eq!(forecast(Units(4), ticks, rate), Some(Forecast {
                requested: Units(0), paid: Units(0), remaining: Units(4),
            }));
        }
    }

    #[test]
    fn unrepresentable_request_is_rejected() {
        assert_eq!(forecast(Units(4), Ticks(u32::MAX), UnitsPerTick(2)), None);
    }

    #[test]
    fn exact_numeric_limit_is_still_a_valid_request() {
        assert_eq!(forecast(Units(u32::MAX), Ticks(u32::MAX), UnitsPerTick(1)),
            Some(Forecast {
                requested: Units(u32::MAX), paid: Units(u32::MAX), remaining: Units(0),
            }));
    }
}
