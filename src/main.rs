use rmcp::{
    handler::server::wrapper::Json, schemars, tool, tool_router, ServiceExt,
};
use serde::Serialize;

/// Response structure for the current_time tool.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CurrentTimeResponse {
    /// ISO 8601 formatted timestamp with timezone offset (e.g. 2026-04-23T14:30:00-04:00)
    pub iso_timestamp: String,
    /// Unix timestamp in seconds
    pub unix_timestamp: i64,
    /// Unix timestamp in milliseconds
    pub unix_timestamp_ms: i64,
    /// Name of the local system timezone (e.g. America/New_York)
    pub timezone: String,
    /// UTC offset in hours (e.g. -4.0)
    pub utc_offset_hours: f32,
    /// UTC offset as a string (e.g. -04:00)
    pub utc_offset: String,
    /// Current date in ISO format (YYYY-MM-DD)
    pub date: String,
    /// Current time in 24-hour format (HH:MM:SS)
    pub time: String,
    /// Day of the week
    pub day_of_week: String,
}

#[derive(Debug, Clone)]
pub struct TimeServer;

#[tool_router(server_handler)]
impl TimeServer {
    #[tool(
        name = "current_time",
        description = "Returns the current local time as an ISO 8601 timestamp, Unix timestamps (seconds and milliseconds), system timezone, UTC offset, date, time, and day of the week. Use this to ground any time-related queries."
    )]
    fn current_time(&self) -> Json<CurrentTimeResponse> {
        let now = chrono::Local::now();
        let offset = now.offset();

        Json(CurrentTimeResponse {
            iso_timestamp: now.to_rfc3339(),
            unix_timestamp: now.timestamp(),
            unix_timestamp_ms: now.timestamp_millis(),
            timezone: chrono::Local::now().format("%Z").to_string(),
            utc_offset_hours: offset.local_minus_utc() as f32 / 3600.0,
            utc_offset: format!("{:+03}:{:02}", offset.local_minus_utc() / 3600, (offset.local_minus_utc().abs() % 3600) / 60),
            date: now.format("%Y-%m-%d").to_string(),
            time: now.format("%H:%M:%S").to_string(),
            day_of_week: now.format("%A").to_string(),
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Time MCP server");

    let service = TimeServer.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
