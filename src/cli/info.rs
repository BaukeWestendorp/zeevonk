use std::path::PathBuf;

use zeevonk::core::gdcs::{Fixture, GeneralizedDmxControlSystem};
use zeevonk::core::showfile::Showfile;

pub fn dump_patch(showfile_path: PathBuf) -> anyhow::Result<()> {
    let showfile = Showfile::load_from_folder(&showfile_path)?;
    let mut gdcs = GeneralizedDmxControlSystem::new();
    gdcs.insert_showfile_data(&showfile)?;

    let mut sorted_fixtures: Vec<&Fixture> = gdcs.fixtures().collect();
    sorted_fixtures.sort_by_key(|f| f.path());

    for fixture in sorted_fixtures {
        dump::dump_fixture(fixture);
    }

    Ok(())
}

mod dump {
    use zeevonk::core::gdcs::{Fixture, FixtureChannelFunctionKind};

    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const DIM: &str = "\x1b[2m";
    const YELLOW: &str = "\x1b[33m";
    const MAGENTA: &str = "\x1b[35m";

    pub fn dump_fixture(fixture: &Fixture) {
        dump_fixture_with_depth(fixture, 0);
    }

    fn dump_fixture_with_depth(fixture: &Fixture, _depth: usize) {
        let path = fixture.path();
        let depth = path.sub_len();

        let guide = if depth == 0 {
            String::new()
        } else {
            format!("{}└─ ", "│   ".repeat(depth.saturating_sub(1)))
        };

        let secondary_indent = format!("{}   ", "│   ".repeat(depth));

        let name = fixture.name();
        let path = fixture.path();

        if path.len() == 1 {
            let base_address = fixture.base_address();
            let fixture_type_id = fixture.gdtf_fixture_type_id();
            let dmx_mode = fixture.gdtf_dmx_mode();

            println!(
                "{guide}{BOLD}{MAGENTA}{name}{RESET} {DIM}({RESET}path{DIM}={RESET}{YELLOW}{path}{RESET}{DIM}, {RESET}base_addr{DIM}={RESET}{YELLOW}{base_address}{RESET}{DIM}){RESET}"
            );

            println!(
                "{secondary_indent}{DIM}type{RESET}={YELLOW}{fixture_type}{RESET}{DIM}, mode{RESET}={YELLOW}{dmx_mode}{RESET}",
                fixture_type = fixture_type_id,
                dmx_mode = dmx_mode,
            );
        } else {
            println!(
                "{guide}{BOLD}{MAGENTA}{name}{RESET} {DIM}({RESET}path{DIM}={RESET}{YELLOW}{path}{RESET}{DIM}){RESET}"
            );
        }

        let channels = fixture.channel_functions().into_iter().collect::<Vec<_>>();
        if channels.is_empty() {
            println!("{secondary_indent}{DIM}<no fixture channels>{RESET}");
        } else {
            for (attribute, fun) in channels {
                let kind = match fun.kind() {
                    FixtureChannelFunctionKind::Physical { addresses } => {
                        addresses.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ")
                    }
                    FixtureChannelFunctionKind::Virtual { relations } => relations
                        .iter()
                        .map(|relation| {
                            format!("{}->{}", relation.fixture_path(), relation.attribute())
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                };
                let min = fun.min();
                let max = fun.max();
                let default = fun.default();
                println!(
                    "{secondary_indent}- {YELLOW}{}{RESET}: [{YELLOW}{}{RESET}], {YELLOW}{}{RESET}..{YELLOW}{}{RESET}, default={YELLOW}{}{RESET}",
                    attribute, kind, min, max, default
                );
            }
        }
    }
}
