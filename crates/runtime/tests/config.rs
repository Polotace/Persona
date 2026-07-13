use std::path::PathBuf;

use persona_runtime::{ConfigError, RuntimeConfig};

const VALID_CONFIG: &str = r#"
schema_version = 1
data_dir = "data"
database_path = "data/persona.db"
log_level = "info"
event_queue_capacity = 64
"#;

#[test]
fn parses_a_supported_runtime_configuration() {
    let config = RuntimeConfig::from_toml(VALID_CONFIG).expect("valid configuration");

    assert_eq!(config.data_dir, PathBuf::from("data"));
    assert_eq!(config.database_path, PathBuf::from("data/persona.db"));
    assert_eq!(config.log_level, "info");
    assert_eq!(config.event_queue_capacity.get(), 64);
}

#[test]
fn rejects_zero_event_queue_capacity() {
    let result = RuntimeConfig::from_toml(
        VALID_CONFIG
            .replace("event_queue_capacity = 64", "event_queue_capacity = 0")
            .as_str(),
    );

    assert!(matches!(result, Err(ConfigError::Invalid(_))));
}

#[test]
fn rejects_unknown_configuration_fields() {
    let result = RuntimeConfig::from_toml(format!("{VALID_CONFIG}unexpected = true").as_str());

    assert!(matches!(result, Err(ConfigError::Invalid(_))));
}
