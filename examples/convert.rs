//! Outil de développement : `cargo run --example convert -- entree.png sortie.ico`
//! (sert notamment à générer `assets/app.ico` avec le moteur de l'application).

use std::path::PathBuf;

use icon_maker::engine::{Settings, pipeline};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args_os().skip(1).map(PathBuf::from);
    let (Some(input), Some(output)) = (args.next(), args.next()) else {
        anyhow::bail!("usage : convert <entree> <sortie.ico>");
    };
    pipeline::convert_file(&input, &output, &Settings::default())
}
