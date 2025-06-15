[dependencies]
# Bestehende Dependencies
rand = "0.8"
config = "0.13"
serde = { version = "1.0", features = ["derive"] }
log = "0.4"
env_logger = "0.10"

# Neue Dependencies für das Web-Interface
rocket = "0.5.0"
rocket_contrib = { version = "0.4.11", features = ["handlebars_templates", "json", "serve"] }
handlebars = "4.3"
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1.28", features = ["full"] }