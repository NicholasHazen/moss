#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SimId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PatchReading {
    id: SimId,
    biomass_units: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TargetView {
    Unchosen,
    Present { id: SimId, biomass_units: u32 },
    Missing { id: SimId },
}

fn view_target(target: Option<SimId>, patches: &[PatchReading]) -> TargetView {
    let id = match target {
        Some(id) => id,
        None => return TargetView::Unchosen,
    };
    match patches.iter().find(|patch| patch.id == id) {
        Some(patch) => TargetView::Present {
            id,
            biomass_units: patch.biomass_units,
        },
        None => TargetView::Missing { id },
    }
}

fn main() {
    let patches = [
        PatchReading { id: SimId(1), biomass_units: 80 },
        PatchReading { id: SimId(3), biomass_units: 0 },
    ];
    println!("{:?}", view_target(Some(SimId(3)), &patches));
    println!("{:?}", view_target(Some(SimId(99)), &patches));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_selected_id_does_not_choose_an_available_patch() {
        let patches = [PatchReading { id: SimId(3), biomass_units: 80 }];
        assert_eq!(view_target(None, &patches), TargetView::Unchosen);
    }

    #[test]
    fn selected_id_is_matched_instead_of_taking_the_first_row() {
        let patches = [
            PatchReading { id: SimId(1), biomass_units: 80 },
            PatchReading { id: SimId(3), biomass_units: 7 },
        ];
        assert_eq!(view_target(Some(SimId(3)), &patches),
            TargetView::Present { id: SimId(3), biomass_units: 7 });
    }

    #[test]
    fn present_zero_is_not_missing() {
        let patches = [PatchReading { id: SimId(3), biomass_units: 0 }];
        assert_eq!(view_target(Some(SimId(3)), &patches),
            TargetView::Present { id: SimId(3), biomass_units: 0 });
    }

    #[test]
    fn missing_target_retains_the_selected_identity() {
        let patches = [PatchReading { id: SimId(3), biomass_units: 80 }];
        assert_eq!(view_target(Some(SimId(99)), &patches),
            TargetView::Missing { id: SimId(99) });
        assert_eq!(view_target(Some(SimId(99)), &[]),
            TargetView::Missing { id: SimId(99) });
    }
}
