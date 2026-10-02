// The `tests` tests, extracted from `ring_family.rs`.
//
// They were inline, and together they were several times the size of the
// code they cover. `super` is still the `ring_family` module, so
// `use super::*` below reaches exactly what it did before the move.
use super::*;
use crate::model::RdkitDescriptors;

fn empty_descriptors() -> RdkitDescriptors {
    RdkitDescriptors::default()
}

#[test]
fn polycyclic_aliphatic_scaffold() {
    let desc = RdkitDescriptors {
        ring_count: Some(4.0),
        aromatic_ring_count: Some(0.0),
        aliphatic_ring_count: Some(4.0),
        fraction_csp3: Some(0.75),
    };
    assert_eq!(
        classify_ring_family(&desc, &[]),
        "natural-product-like polycyclic scaffold"
    );
}

#[test]
fn polyaromatic_scaffold() {
    let desc = RdkitDescriptors {
        ring_count: Some(3.0),
        aromatic_ring_count: Some(3.0),
        aliphatic_ring_count: Some(0.0),
        fraction_csp3: Some(0.10),
    };
    assert_eq!(classify_ring_family(&desc, &[]), "polyaromatic scaffold");
}

#[test]
fn motif_labels_override_descriptor_heuristics() {
    assert_eq!(
        classify_ring_family(&empty_descriptors(), &["Flavone ring".to_string()]),
        "flavonoid-like scaffold"
    );
    assert_eq!(
        classify_ring_family(&empty_descriptors(), &["Indole ring".to_string()]),
        "fused heteroaromatic scaffold"
    );
    assert_eq!(
        classify_ring_family(
            &empty_descriptors(),
            &["Sugar-like oxygen ring".to_string()]
        ),
        "sugar-like oxygenated ring system"
    );
}
