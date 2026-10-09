# time-mcp

A minimal [Model Context Protocol (MCP)](https://modelcontextprotocol.io) server written in Rust that exposes the current local time.

It speaks stdio and supports MCP protocol revisions `2024-11-05` through `2026-07-28`.

## Tool: `current_time`

Returns structured time data for grounding timestamp queries. It is read-only and has no side effects.

| Field              | Description                                                                 |
|--------------------|-----------------------------------------------------------------------------|
| `iso_timestamp`    | ISO 8601 timestamp with timezone offset                                     |
| `unix_timestamp`   | Unix timestamp in seconds                                                   |
| `unix_timestamp_ms`| Unix timestamp in milliseconds                                              |
| `timezone`         | IANA timezone name (e.g., `America/New_York`), or the UTC offset if the OS does not report one |
| `utc_offset_hours` | UTC offset as a float (e.g., `-4.0`)                                        |
| `utc_offset`       | UTC offset string (e.g., `-04:00`)                                          |
| `date`             | Current date (`YYYY-MM-DD`)                                                 |
| `time`             | Current time (`HH:MM:SS`)                                                   |
| `day_of_week`      | Day of the week (e.g., `Thursday`)                                          |

## Installation

### From crates.io

```bash
cargo install time-mcp
```

### From source

```bash
cargo install --path .
```

## Client configuration

The server is launched over stdio with the `time-mcp` binary. Make sure it is on your `PATH` (for example, after `cargo install`).

### Generic MCP client

```json
{
  "mcpServers": {
    "time": {
      "command": "time-mcp"
    }
  }
}
```

### opencode

Add to `opencode.json` (project) or your global opencode config:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "time": {
      "type": "local",
      "command": ["time-mcp"],
      "enabled": true
    }
  }
}
```

Check with `opencode mcp list`.

### Codex

```bash
codex mcp add time -- time-mcp
```

This writes the following to `~/.codex/config.toml`:

```toml
[mcp_servers.time]
command = "time-mcp"
```

Check with `codex mcp list`.

### pi

```bash
pi mcp add time -- time-mcp
```

This writes the following to `~/.pi/agent/mcp.json`:

```json
{
  "mcpServers": {
    "time": {
      "command": "time-mcp"
    }
  }
}
```

Check with `pi mcp list`.

## Development

```bash
cargo build --locked
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
```

The minimum supported Rust version is 1.88, the minimum the `rmcp` SDK requires.

The integration tests in `tests/stdio_protocol.rs` run the compiled binary over stdio and check the handshake, version negotiation, tool listing, and tool output.

## License

MIT License. See [LICENSE](./LICENSE).
