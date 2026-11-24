// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smellfish-rs project

use crate::evidence::classify_ring_family;
use crate::model::{RdkitDescriptors, RingFamily};

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
        classify_ring_family(&empty, &["Flavone ring".to_string()]),
        RingFamily::Flavonoid
    );
    assert_eq!(
        classify_ring_family(&empty, &["Indole ring".to_string()]),
        RingFamily::Heteroaromatic
    );
    assert_eq!(
        classify_ring_family(&empty, &["Sugar-like oxygen ring".to_string()]),
        RingFamily::Sugar
    );
    assert_eq!(
        classify_ring_family(&empty, &["Steroid nucleus".to_string()]),
        RingFamily::Steroid
    );
    assert_eq!(
        classify_ring_family(&empty, &["Macrolide core".to_string()]),
        RingFamily::Macrolide
    );
}
