//! Drives the compiled server over stdio, the same way MCP clients launch it.

use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

const CURRENT_VERSION: &str = "2026-07-28";
const TOOL_OUTPUT_KEYS: [&str; 9] = [
    "iso_timestamp",
    "unix_timestamp",
    "unix_timestamp_ms",
    "timezone",
    "utc_offset_hours",
    "utc_offset",
    "date",
    "time",
    "day_of_week",
];

/// The per-request `_meta` that stateless (2026-07-28) clients send instead of an `initialize` handshake.
fn stateless_meta() -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": CURRENT_VERSION,
        "io.modelcontextprotocol/clientCapabilities": {},
        "io.modelcontextprotocol/clientInfo": { "name": "stdio-protocol-test", "version": "0.0.0" },
    })
}

struct Server {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Server {
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_time-mcp-server"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("server binary should start");
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = BufReader::new(child.stdout.take().expect("piped stdout"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, message: Value) {
        writeln!(self.stdin, "{message}").expect("write to server stdin");
        self.stdin.flush().expect("flush server stdin");
    }

    fn recv(&mut self) -> Value {
        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .expect("read from server stdout");
        serde_json::from_str(&line)
            .unwrap_or_else(|e| panic!("invalid JSON-RPC line {line:?}: {e}"))
    }

    fn request(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let response = self.recv();
        assert_eq!(response["id"], id, "response id should match request");
        assert!(
            response.get("error").is_none(),
            "unexpected error: {response}"
        );
        response["result"].clone()
    }

    /// Runs the 2025-era handshake and returns the `initialize` result.
    fn initialize(&mut self, protocol_version: &str) -> Value {
        let result = self.request(
            1,
            "initialize",
            json!({
                "protocolVersion": protocol_version,
                "capabilities": {},
                "clientInfo": { "name": "stdio-protocol-test", "version": "0.0.0" },
            }),
        );
        self.send(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        result
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn initialize_reports_the_server_identity_and_tools_capability() {
    let mut server = Server::spawn();
    let result = server.initialize("2025-11-25");

    assert_eq!(result["protocolVersion"], "2025-11-25");
    assert_eq!(result["serverInfo"]["name"], "time-mcp-server");
    assert_eq!(result["serverInfo"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(result["serverInfo"]["title"], "Time MCP Server");
    assert!(result["capabilities"]["tools"].is_object());
    assert!(
        result["instructions"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
    );
}

#[test]
fn negotiates_supported_legacy_protocol_versions() {
    for version in ["2025-06-18", "2025-03-26", "2024-11-05"] {
        let mut server = Server::spawn();
        let result = server.initialize(version);
        assert_eq!(result["protocolVersion"], version);
    }
}

#[test]
fn answers_unknown_protocol_versions_with_a_supported_one() {
    let mut server = Server::spawn();
    let result = server.initialize("2099-01-01");
    let negotiated = result["protocolVersion"].as_str().unwrap();
    assert!(
        [
            "2024-11-05",
            "2025-03-26",
            "2025-06-18",
            "2025-11-25",
            CURRENT_VERSION
        ]
        .contains(&negotiated),
        "server should answer with a version it supports, got {negotiated}"
    );
}

#[test]
fn lists_current_time_with_annotations_and_output_schema() {
    let mut server = Server::spawn();
    server.initialize("2025-11-25");
    let result = server.request(2, "tools/list", json!({}));

    let tools = result["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 1);
    let tool = &tools[0];

    assert_eq!(tool["name"], "current_time");
    assert_eq!(tool["title"], "Current Time");
    assert!(tool["description"].as_str().is_some_and(|s| !s.is_empty()));
    assert_eq!(tool["inputSchema"]["type"], "object");

    let annotations = &tool["annotations"];
    assert_eq!(annotations["readOnlyHint"], true);
    assert_eq!(annotations["destructiveHint"], false);
    assert_eq!(annotations["openWorldHint"], false);

    let output = &tool["outputSchema"];
    assert_eq!(output["type"], "object");
    for key in TOOL_OUTPUT_KEYS {
        assert!(
            output["properties"][key].is_object(),
            "outputSchema missing {key}"
        );
    }
}

#[test]
fn current_time_returns_consistent_structured_content() {
    let mut server = Server::spawn();
    server.initialize("2025-11-25");
    let result = server.request(
        2,
        "tools/call",
        json!({ "name": "current_time", "arguments": {} }),
    );

    assert_eq!(result["isError"], false);
    let content = &result["structuredContent"];
    for key in TOOL_OUTPUT_KEYS {
        assert!(!content[key].is_null(), "structuredContent missing {key}");
    }

    let seconds = content["unix_timestamp"].as_i64().unwrap();
    let millis = content["unix_timestamp_ms"].as_i64().unwrap();
    assert!(
        (millis - seconds * 1000).abs() < 1000,
        "ms and s timestamps disagree"
    );

    let offset = content["utc_offset"].as_str().unwrap();
    assert_eq!(
        offset.len(),
        6,
        "offset should look like ±HH:MM, got {offset}"
    );
    assert!(offset.starts_with('+') || offset.starts_with('-'));

    let date = content["date"].as_str().unwrap();
    let time = content["time"].as_str().unwrap();
    let iso = content["iso_timestamp"].as_str().unwrap();
    assert!(
        iso.starts_with(&format!("{date}T{time}")),
        "iso {iso} should match {date} {time}"
    );
    assert!(
        iso.ends_with(offset),
        "iso {iso} should end with the UTC offset {offset}"
    );
}

#[test]
fn answers_server_discover_for_the_stateless_revision() {
    let mut server = Server::spawn();
    let result = server.request(1, "server/discover", json!({ "_meta": stateless_meta() }));

    let supported = result["supportedVersions"].as_array().unwrap();
    assert!(supported.iter().any(|v| v == CURRENT_VERSION));
    assert!(supported.iter().any(|v| v == "2025-11-25"));
    assert!(result["capabilities"]["tools"].is_object());
    assert_eq!(
        result["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
        "time-mcp-server"
    );
}

#[test]
fn serves_stateless_requests_without_a_handshake() {
    let mut server = Server::spawn();
    let meta = stateless_meta();

    let listed = server.request(1, "tools/list", json!({ "_meta": meta }));
    assert_eq!(listed["tools"][0]["name"], "current_time");
    assert_eq!(listed["resultType"], "complete");

    let called = server.request(
        2,
        "tools/call",
        json!({ "name": "current_time", "arguments": {}, "_meta": meta }),
    );
    assert_eq!(called["isError"], false);
    assert_eq!(called["resultType"], "complete");
    assert!(called["structuredContent"]["iso_timestamp"].is_string());
}
