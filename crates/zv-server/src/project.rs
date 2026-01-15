use theymx::UniverseId;

pub enum OutputInstanceDefinition {
    EnttecOpenDmx { universe_id: UniverseId, serial_number: String },
}
