//! The manifest gray reads at install and sidecar startup.

pub fn manifest() -> serde_json::Value {
    serde_json::json!({
        "name": "maker",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "1.1",
        "tools": [],
        "commands": ["/maker"],
        "completion": ["new", "check", "build", "release", "publish", "ship"],
    })
}
