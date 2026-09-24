#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct State {
    completed_ticks: u64,
    reserve_units: u32,
}

fn initial_state() -> State {
    State { completed_ticks: 0, reserve_units: 60 }
}

fn advance(state: &mut State) {
    state.reserve_units = state.reserve_units.saturating_sub(1);
    state.completed_ticks = state.completed_ticks
        .checked_add(1).expect("tick counter overflow");
}

fn inspect(state: &State) -> (u64, u32) {
    (state.completed_ticks, state.reserve_units)
}

fn run(reads_per_tick: usize) -> State {
    let mut state = initial_state();
    for _ in 0..3 {
        advance(&mut state);
        for _ in 0..reads_per_tick {
            assert_eq!(inspect(&state).0, state.completed_ticks);
        }
    }
    state
}

fn main() {
    let state = run(200);
    println!("ticks={} reserve={}", state.completed_ticks, state.reserve_units);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observations_do_not_advance_time() {
        let state = initial_state();
        for _ in 0..120 {
            assert_eq!(inspect(&state), (0, 60));
        }
        assert_eq!(state, initial_state());
    }

    #[test]
    fn observation_frequency_does_not_change_outcome() {
        assert_eq!(run(2), run(200));
        assert_eq!(inspect(&run(2)), (3, 57));
    }

    #[test]
    fn empty_reserve_still_completes_a_tick() {
        let mut state = State { completed_ticks: 8, reserve_units: 0 };
        advance(&mut state);
        assert_eq!(inspect(&state), (9, 0));
    }
}
