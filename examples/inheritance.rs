const ALGORITHM: &str = "moss-traits-lcg64-v1";
const MIN_STRIDE: u8 = 1;
const MAX_STRIDE: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Key { run: u32, id: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Parent { key: Key, stride_cells_per_action: u8 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Origin {
    child: Key,
    parents: [Key; 2],
    parent_strides: [u8; 2],
    mutation_enabled: bool,
    source_parent: Key,
    inherited: u8,
    proposed_delta: i8,
    applied_delta: i8,
    expressed: u8,
    seed: u64,
    algorithm: &'static str,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Rejected { Identity, ParentTrait }
struct Rng { state: u64 }
impl Rng {
    fn new(seed: u64) -> Self { Self { state: seed } }
    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.state >> 32) as u32
    }
}
fn bounded_stride(inherited: u8, delta: i8) -> u8 {
    (i16::from(inherited) + i16::from(delta))
        .clamp(i16::from(MIN_STRIDE), i16::from(MAX_STRIDE)) as u8
}

fn inherit(parents: &[Parent; 2], child: Key, seed: u64, mutate: bool)
    -> Result<Origin, Rejected>
{
    if child.id == 0 || parents[0].key.id == 0 || parents[1].key.id == 0
        || parents[0].key == parents[1].key
        || parents.iter().any(|parent| parent.key.run != child.run || parent.key == child) {
        return Err(Rejected::Identity);
    }
    if parents.iter().any(|parent| !(MIN_STRIDE..=MAX_STRIDE)
        .contains(&parent.stride_cells_per_action)) { return Err(Rejected::ParentTrait); }
    let mut ordered = *parents;
    ordered.sort_by_key(|parent| parent.key);
    let mut rng = Rng::new(seed);
    let source = ordered[(rng.next_u32() % 2) as usize];
    let mutation_draw = (rng.next_u32() % 3) as i8 - 1;
    let proposed_delta = if mutate { mutation_draw } else { 0 };
    let inherited = source.stride_cells_per_action;
    let expressed = bounded_stride(inherited, proposed_delta);
    Ok(Origin { child, parents: [ordered[0].key, ordered[1].key],
        parent_strides: [ordered[0].stride_cells_per_action, ordered[1].stride_cells_per_action],
        mutation_enabled: mutate, source_parent: source.key, inherited, proposed_delta,
        applied_delta: expressed as i8 - inherited as i8, expressed,
        seed, algorithm: ALGORITHM })
}
fn parents() -> [Parent; 2] {
    [Parent { key: Key { run: 7, id: 2 }, stride_cells_per_action: 6 },
     Parent { key: Key { run: 7, id: 1 }, stride_cells_per_action: 2 }]
}
fn main() {
    let parents = parents();
    for (index, seed) in [3, 5, 6].into_iter().enumerate() {
        let origin = inherit(&parents, Key { run: 7, id: 100 + index as u32 }, seed, true).unwrap();
        println!("seed={} child={} source={} inherited={} proposed={} applied={} expressed={}",
            seed, origin.child.id, origin.source_parent.id, origin.inherited,
            origin.proposed_delta, origin.applied_delta, origin.expressed);
    }
    let copied = inherit(&parents, Key { run: 7, id: 103 }, 6, false).unwrap();
    println!("copy-only={} algorithm={}", copied.expressed, copied.algorithm);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn child_origin_records_copy_mutation_and_independent_identity() {
        let origin = inherit(&parents(), Key { run: 7, id: 100 }, 6, true).unwrap();
        assert_eq!(origin, Origin { child: Key { run: 7, id: 100 },
            parents: [Key { run: 7, id: 1 }, Key { run: 7, id: 2 }],
            parent_strides: [2, 6], mutation_enabled: true, source_parent: Key { run: 7, id: 1 }, inherited: 2,
            proposed_delta: 1, applied_delta: 1, expressed: 3,
            seed: 6, algorithm: "moss-traits-lcg64-v1" });
    }
    #[test]
    fn parents_are_unchanged_and_input_order_does_not_change_inheritance() {
        let original = parents();
        let before = original;
        let a = inherit(&original, Key { run: 7, id: 100 }, 3, true).unwrap();
        let b = inherit(&[original[1], original[0]], Key { run: 7, id: 100 }, 3, true).unwrap();
        assert_eq!(original, before);
        assert_eq!(a, b);
    }
    #[test]
    fn mutation_disabled_preserves_the_selected_parent_value() {
        let copied = inherit(&parents(), Key { run: 7, id: 100 }, 6, false).unwrap();
        let changed = inherit(&parents(), Key { run: 7, id: 100 }, 6, true).unwrap();
        assert_eq!((copied.inherited, copied.expressed, copied.proposed_delta, copied.applied_delta),
            (2, 2, 0, 0));
        assert_eq!(copied.source_parent, changed.source_parent);
        assert_ne!(copied.expressed, changed.expressed);
    }
    #[test]
    fn literal_generator_outputs_pin_the_algorithm_and_draw_order() {
        let mut rng = Rng::new(0);
        assert_eq!([rng.next_u32(), rng.next_u32(), rng.next_u32()],
            [335_903_614, 436_792_849, 2_599_843_874]);
    }
    #[test]
    fn clamping_records_the_effective_delta_at_both_bounds() {
        assert_eq!(bounded_stride(1, -1), 1);
        assert_eq!(bounded_stride(8, 1), 8);
        for stride in [MIN_STRIDE, MAX_STRIDE] {
            let p = [Parent { key: Key { run: 7, id: 1 }, stride_cells_per_action: stride },
                Parent { key: Key { run: 7, id: 2 }, stride_cells_per_action: stride }];
            for seed in 0..64 {
                let origin = inherit(&p, Key { run: 7, id: 100 }, seed, true).unwrap();
                assert!((MIN_STRIDE..=MAX_STRIDE).contains(&origin.expressed));
                assert_eq!(i16::from(origin.inherited) + i16::from(origin.applied_delta),
                    i16::from(origin.expressed));
                if (stride == 1 && origin.proposed_delta == -1)
                    || (stride == 8 && origin.proposed_delta == 1) {
                    assert_eq!(origin.applied_delta, 0);
                }
            }
        }
    }
    #[test]
    fn invalid_identity_and_out_of_bounds_parents_are_rejected() {
        let p = parents();
        assert_eq!(inherit(&p, Key { run: 7, id: 1 }, 0, true), Err(Rejected::Identity));
        assert_eq!(inherit(&p, Key { run: 8, id: 100 }, 0, true), Err(Rejected::Identity));
        assert_eq!(inherit(&[p[0], p[0]], Key { run: 7, id: 100 }, 0, true), Err(Rejected::Identity));
        let mut invalid = p; invalid[0].stride_cells_per_action = 0;
        assert_eq!(inherit(&invalid, Key { run: 7, id: 100 }, 0, true), Err(Rejected::ParentTrait));
    }
    #[test]
    fn repeated_inputs_repeat_but_different_seeds_need_not_make_unique_children() {
        let p = parents();
        for seed in 0..64 {
            assert_eq!(inherit(&p, Key { run: 7, id: 100 }, seed, true),
                inherit(&p, Key { run: 7, id: 100 }, seed, true));
        }
        let traits: std::collections::BTreeSet<_> = (0..64).map(|seed|
            inherit(&p, Key { run: 7, id: 100 }, seed, true).unwrap().expressed).collect();
        assert!(traits.len() < 64);
    }
    #[test]
    fn specified_seeds_can_decrease_or_increase_the_inherited_trait() {
        let lower = inherit(&parents(), Key { run: 7, id: 100 }, 3, true).unwrap();
        let higher = inherit(&parents(), Key { run: 7, id: 101 }, 5, true).unwrap();
        assert_eq!((lower.inherited, lower.proposed_delta, lower.expressed), (2, -1, 1));
        assert_eq!((higher.inherited, higher.proposed_delta, higher.expressed), (6, 1, 7));
    }

}
