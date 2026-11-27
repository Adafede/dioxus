// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::evidence::ring_family::classify_ring_family;
use crate::model::{RdkitDescriptors, RdkitMotifHit, RingFamily, SourceClass};

fn hit(label: &str) -> [RdkitMotifHit; 1] {
    [RdkitMotifHit {
        label: label.to_string(),
        source_class: SourceClass::Unknown,
        kingdom: String::new(),
        kingdoms: Vec::new(),
    }]
}

#[test]
fn descriptor_heuristics_classify_ring_families() {
    let polycyclic = RdkitDescriptors {
        ring_count: Some(4.0),
        aromatic_ring_count: Some(0.0),
        aliphatic_ring_count: Some(4.0),
        fraction_csp3: Some(0.75),
    };
    assert_eq!(
        classify_ring_family(&polycyclic, &[]),
        RingFamily::Polycyclic
    );

    let polyaromatic = RdkitDescriptors {
        ring_count: Some(3.0),
        aromatic_ring_count: Some(3.0),
        aliphatic_ring_count: Some(0.0),
        fraction_csp3: Some(0.10),
    };
    assert_eq!(
        classify_ring_family(&polyaromatic, &[]),
        RingFamily::Polyaromatic
    );

    let monocycle = RdkitDescriptors {
        ring_count: Some(1.0),
        aromatic_ring_count: Some(0.0),
        aliphatic_ring_count: Some(1.0),
        fraction_csp3: Some(1.0),
    };
    assert_eq!(
        classify_ring_family(&monocycle, &[]),
        RingFamily::CompactRing
    );

    let mixed = RdkitDescriptors {
        ring_count: Some(2.0),
        aromatic_ring_count: Some(1.0),
        aliphatic_ring_count: Some(1.0),
        fraction_csp3: Some(0.3),
    };
    assert_eq!(
        classify_ring_family(&mixed, &[]),
        RingFamily::MixedAromaticAliphatic
    );
    assert_eq!(
        classify_ring_family(&RdkitDescriptors::default(), &[]),
        RingFamily::Acyclic
    );
}

#[test]
fn motif_labels_override_descriptor_heuristics() {
    let empty = RdkitDescriptors::default();
    assert_eq!(
        classify_ring_family(&empty, &hit("Flavone ring")),
        RingFamily::Flavonoid
    );
    assert_eq!(
        classify_ring_family(&empty, &hit("Indole ring")),
        RingFamily::Heteroaromatic
    );
    assert_eq!(
        classify_ring_family(&empty, &hit("Sugar-like oxygen ring")),
        RingFamily::Sugar
    );
    assert_eq!(
        classify_ring_family(&empty, &hit("Steroid nucleus")),
        RingFamily::Steroid
    );
    assert_eq!(
        classify_ring_family(&empty, &hit("Macrolide core")),
        RingFamily::Macrolide
    );
}
