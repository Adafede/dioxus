// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the smarts-evoliposuction project

//! Release-mode benchmark smoke harness for `smarts-evoliposuction` SDF/split workloads.

use std::fmt::Write as _;
use std::hint::black_box;
use std::io::Write as _;
use std::time::Instant;

use smarts_evoliposuction::{SplitConfig, parse_lmsd_tsv, sdf_zip_to_tsv, split_dataset};
use tempfile::NamedTempFile;
use zip::write::SimpleFileOptions;

fn measure<T>(name: &str, mut f: impl FnMut() -> T) {
    drop(black_box(f()));
    let mut times_ms = [0.0f64; 3];
    for slot in &mut times_ms {
        let start = Instant::now();
        drop(black_box(f()));
        *slot = start.elapsed().as_secs_f64() * 1000.0;
    }
    times_ms.sort_by(f64::total_cmp);
    println!("{name:<34} {:>9.3} ms", times_ms[1]);
}

fn main() {
    let mut sdf = String::with_capacity(2000 * 500);
    for i in 0..2000 {
        let fam = if i % 2 == 0 { "FA" } else { "GL" };
        let sub = format!("{fam}0{}", (i % 4) + 1);
        let _ = write!(
            sdf,
            "  CDK     2DS\n\n  1 0  0  0  0  0  0  0  0  0999 V2000\n    0.0 0.0 0.0 C 0 0 0\nM  END\n\
             > <LM_ID>\nLM{fam}{i:08}\n\n> <NAME>\nLipid_{i}\n\n> <CATEGORY>\nCat_{fam}\n\n\
             > <MAIN_CLASS>\nMain_{fam}\n\n> <SUB_CLASS>\n{sub}\n\n> <SMILES>\n{}C(=O)O\n\n$$$$\n",
            "C".repeat((i % 20) + 2)
        );
    }

    let tmp_zip = NamedTempFile::new().unwrap_or_else(|_| std::process::abort());
    {
        let mut zip = zip::ZipWriter::new(&tmp_zip);
        let opts =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        if zip.start_file("LMSD.sdf", opts).is_err()
            || zip.write_all(sdf.as_bytes()).is_err()
            || zip.finish().is_err()
        {
            std::process::abort();
        }
    }

    let (tsv, _) = sdf_zip_to_tsv(tmp_zip.path()).unwrap_or_default();

    measure("evoliposuction_sdf_zip_2k_to_tsv", || {
        sdf_zip_to_tsv(tmp_zip.path()).map_or(0, |(_, c)| c)
    });
    measure("evoliposuction_split_2k_tsv", || {
        parse_lmsd_tsv(&tsv).map_or(0, |d| {
            split_dataset(&d, &SplitConfig::default()).classes.len()
        })
    });
}
