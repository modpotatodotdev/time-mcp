# time-mcp-server

A minimal [Model Context Protocol (MCP)](https://modelcontextprotocol.io) server written in Rust that exposes the current local time.

## Tool: `current_time`

Returns structured time data for grounding timestamp queries:

| Field              | Description                                      |
|--------------------|--------------------------------------------------|
| `iso_timestamp`    | ISO 8601 timestamp with timezone offset          |
| `unix_timestamp`   | Unix timestamp in seconds                        |
| `unix_timestamp_ms`| Unix timestamp in milliseconds                   |
| `timezone`         | Local system timezone abbreviation               |
| `utc_offset_hours` | UTC offset as a float (e.g., `-4.0`)             |
| `utc_offset`       | UTC offset string (e.g., `-04:00`)               |
| `date`             | Current date (`YYYY-MM-DD`)                      |
| `time`             | Current time (`HH:MM:SS`)                        |
| `day_of_week`      | Day of the week (e.g., `Thursday`)               |

## Installation

Add to your MCP client configuration:

```json
{
  "mcpServers": {
    "time": {
      "command": "time-mcp-server"
    }
  }
}
```

Or install from source:

```bash
cargo install --path .
```

## Development

```bash
cargo build --release
cargo test
```

## License

MIT License. See [LICENSE](./LICENSE).
