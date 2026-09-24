const MAX_WINDOW_TICKS: u64 = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReadingWindow {
    start_tick: u64,
    end_tick_exclusive: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WindowError {
    InvalidInteger { field: &'static str },
    CountOutOfRange { tick_count: u64 },
    EndOverflow { start_tick: u64, tick_count: u64 },
}

impl ReadingWindow {
    fn new(start_tick: u64, tick_count: u64) -> Result<Self, WindowError> {
        if !(1..=MAX_WINDOW_TICKS).contains(&tick_count) {
            return Err(WindowError::CountOutOfRange { tick_count });
        }
        let end_tick_exclusive = start_tick.checked_add(tick_count)
            .ok_or(WindowError::EndOverflow { start_tick, tick_count })?;
        Ok(Self { start_tick, end_tick_exclusive })
    }

    fn start_tick(self) -> u64 { self.start_tick }
    fn end_tick_exclusive(self) -> u64 { self.end_tick_exclusive }
    fn tick_count(self) -> u64 { self.end_tick_exclusive - self.start_tick }
}

fn parse_window(start: &str, count: &str) -> Result<ReadingWindow, WindowError> {
    let start_tick = start.parse::<u64>()
        .map_err(|_| WindowError::InvalidInteger { field: "start tick" })?;
    let tick_count = count.parse::<u64>()
        .map_err(|_| WindowError::InvalidInteger { field: "tick count" })?;
    ReadingWindow::new(start_tick, tick_count)
}

fn main() {
    let window = parse_window("12", "3").expect("the example window is valid");
    println!("ticks {}..{} ({} ticks)",
        window.start_tick(), window.end_tick_exclusive(), window.tick_count());
    match parse_window("12", "0") {
        Ok(_) => println!("accepted"),
        Err(error) => println!("rejected: {error:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_supported_count_boundaries_are_accepted() {
        for count in [1, MAX_WINDOW_TICKS] {
            let window = ReadingWindow::new(12, count).unwrap();
            assert_eq!(window.start_tick(), 12);
            assert_eq!(window.end_tick_exclusive(), 12 + count);
            assert_eq!(window.tick_count(), count);
        }
    }

    #[test]
    fn zero_and_oversized_counts_are_rejected_not_clamped() {
        for count in [0, MAX_WINDOW_TICKS + 1] {
            assert_eq!(ReadingWindow::new(12, count),
                Err(WindowError::CountOutOfRange { tick_count: count }));
        }
    }

    #[test]
    fn exact_end_limit_is_valid_but_overflow_is_not() {
        let window = ReadingWindow::new(u64::MAX - 1, 1).unwrap();
        assert_eq!(window.end_tick_exclusive(), u64::MAX);
        assert_eq!(ReadingWindow::new(u64::MAX, 1),
            Err(WindowError::EndOverflow { start_tick: u64::MAX, tick_count: 1 }));
    }

    #[test]
    fn parse_errors_identify_the_field_that_failed() {
        assert_eq!(parse_window("fern", "3"),
            Err(WindowError::InvalidInteger { field: "start tick" }));
        for text in ["three", "-1", "18446744073709551616"] {
            assert_eq!(parse_window("12", text),
                Err(WindowError::InvalidInteger { field: "tick count" }));
        }
    }

    #[test]
    fn parsed_integers_still_pass_through_domain_validation() {
        assert_eq!(parse_window("12", "0"),
            Err(WindowError::CountOutOfRange { tick_count: 0 }));
        assert_eq!(parse_window("12", "4097"),
            Err(WindowError::CountOutOfRange { tick_count: 4097 }));
        assert_eq!(parse_window("12", "3").unwrap().end_tick_exclusive(), 15);
    }
}
