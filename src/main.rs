use chrono::{DateTime, Local, TimeZone};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::wrapper::Json,
    model::{Implementation, ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router,
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
    /// IANA name of the local system timezone (e.g. America/New_York). Falls back to the UTC offset when the OS does not report one.
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

/// Builds the current_time response for `now`, reporting every field in that moment's local offset.
fn describe_time<Tz: TimeZone>(now: DateTime<Tz>, timezone: String) -> CurrentTimeResponse {
    let local = now.fixed_offset();
    let offset_secs = local.offset().local_minus_utc();

    CurrentTimeResponse {
        iso_timestamp: local.to_rfc3339(),
        unix_timestamp: now.timestamp(),
        unix_timestamp_ms: now.timestamp_millis(),
        timezone,
        utc_offset_hours: offset_secs as f32 / 3600.0,
        utc_offset: format_utc_offset(offset_secs),
        date: local.format("%Y-%m-%d").to_string(),
        time: local.format("%H:%M:%S").to_string(),
        day_of_week: local.format("%A").to_string(),
    }
}

/// Formats a UTC offset given in seconds as `±HH:MM`, keeping the sign when the offset is under an hour.
fn format_utc_offset(offset_secs: i32) -> String {
    let sign = if offset_secs < 0 { '-' } else { '+' };
    let abs = offset_secs.unsigned_abs();
    format!("{sign}{:02}:{:02}", abs / 3600, (abs % 3600) / 60)
}

/// Name of the system timezone, or the UTC offset when the OS does not report an IANA name.
fn system_timezone(utc_offset: &str) -> String {
    iana_time_zone::get_timezone().unwrap_or_else(|_| utc_offset.to_string())
}

#[derive(Debug, Clone)]
pub struct TimeServer;

#[tool_router]
impl TimeServer {
    #[tool(
        name = "current_time",
        title = "Current Time",
        description = "Returns the current local time as an ISO 8601 timestamp, Unix timestamps (seconds and milliseconds), system timezone, UTC offset, date, time, and day of the week. Use this to ground any time-related queries.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn current_time(&self) -> Json<CurrentTimeResponse> {
        let now = Local::now();
        let utc_offset = format_utc_offset(now.offset().local_minus_utc());
        Json(describe_time(now, system_timezone(&utc_offset)))
    }
}

#[tool_handler]
impl ServerHandler for TimeServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                    .with_title("Time MCP Server")
                    .with_description(env!("CARGO_PKG_DESCRIPTION"))
                    .with_website_url(env!("CARGO_PKG_REPOSITORY")),
            )
            .with_instructions(
                "Call current_time to ground any question about the current date, time, or timezone.",
            )
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    #[test]
    fn formats_utc_offsets_with_sign_and_minutes() {
        assert_eq!(format_utc_offset(0), "+00:00");
        assert_eq!(format_utc_offset(-4 * 3600), "-04:00");
        assert_eq!(format_utc_offset(5 * 3600 + 45 * 60), "+05:45");
        assert_eq!(format_utc_offset(-(9 * 3600 + 30 * 60)), "-09:30");
        // Offsets under an hour must keep their sign.
        assert_eq!(format_utc_offset(-30 * 60), "-00:30");
    }

    #[test]
    fn describes_time_in_the_given_offset() {
        let now = FixedOffset::west_opt(4 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 4, 23, 14, 30, 0)
            .unwrap();

        let response = describe_time(now, "America/New_York".to_string());

        assert_eq!(response.iso_timestamp, "2026-04-23T14:30:00-04:00");
        assert_eq!(response.unix_timestamp, 1_776_969_000);
        assert_eq!(response.unix_timestamp_ms, 1_776_969_000_000);
        assert_eq!(response.timezone, "America/New_York");
        assert_eq!(response.utc_offset_hours, -4.0);
        assert_eq!(response.utc_offset, "-04:00");
        assert_eq!(response.date, "2026-04-23");
        assert_eq!(response.time, "14:30:00");
        assert_eq!(response.day_of_week, "Thursday");
    }

    #[test]
    fn reports_sub_hour_offsets_as_fractional_hours() {
        let now = FixedOffset::west_opt(30 * 60)
            .unwrap()
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .unwrap();

        let response = describe_time(now, "-00:30".to_string());

        assert_eq!(response.utc_offset_hours, -0.5);
        assert_eq!(response.utc_offset, "-00:30");
        assert_eq!(now.offset().local_minus_utc(), -30 * 60);
    }
}
