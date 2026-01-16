use std::path::PathBuf;

use zeevonk::attr::{Attribute, CustomName};
use zeevonk::project::definition::ProjectDefinition;
use zeevonk::project::patch::{FixtureId, FixtureIdPart};
use zeevonk::server::Server;
use zeevonk::value::AttributeValues;

/// Runs the project at the given path.
pub fn run_project(project_path: PathBuf) -> anyhow::Result<()> {
    let project_definition = ProjectDefinition::load_from_folder(&project_path)?;
    let server = Server::new(project_definition)?;
    server.start();

    let mut values = AttributeValues::new();
    values.set(
        FixtureId::new(FixtureIdPart::new(1).unwrap())
            .extended_with(FixtureIdPart::new(1).unwrap()),
        Attribute::Dimmer,
        127.0,
    );
    values.set(
        FixtureId::new(FixtureIdPart::new(1).unwrap())
            .extended_with(FixtureIdPart::new(1).unwrap()),
        Attribute::Ctc,
        0.0,
    );
    values.set(
        FixtureId::new(FixtureIdPart::new(1).unwrap())
            .extended_with(FixtureIdPart::new(1).unwrap()),
        Attribute::Tint,
        0.5,
    );
    values.set(
        FixtureId::new(FixtureIdPart::new(1).unwrap())
            .extended_with(FixtureIdPart::new(1).unwrap()),
        Attribute::Custom(CustomName::new("Color XF".to_string())),
        0.0,
    );

    server.test_send(values);

    loop {
        std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / 60.0));
    }
}
