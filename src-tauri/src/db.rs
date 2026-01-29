use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use chrono::{DateTime, Utc, Local};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UsageDay {
    pub date: String,
    pub total_minutes: u32,
    pub sessions: Vec<UsageSession>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UsageSession {
    pub start: String,
    pub end: String,
    pub start_ts: i64,
    pub end_ts: i64,
    pub duration_minutes: u32,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn init(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS usage_pings (
                timestamp INTEGER PRIMARY KEY
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn record_ping(&self) -> Result<()> {
        let now = Utc::now().timestamp();
        self.conn.execute(
            "INSERT OR IGNORE INTO usage_pings (timestamp) VALUES (?)",
            params![now],
        )?;
        Ok(())
    }

    pub fn get_usage_for_range(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>, threshold_seconds: u32) -> Result<Vec<UsageDay>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp FROM usage_pings WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp ASC"
        )?;
        let pings: Vec<i64> = stmt.query_map(params![start_date.timestamp(), end_date.timestamp()], |row| row.get(0))?
            .collect::<Result<Vec<_>>>()?;

        // Group pings by day and find sessions
        let mut days: std::collections::BTreeMap<String, Vec<i64>> = std::collections::BTreeMap::new();
        for ping in pings {
            let dt = DateTime::from_timestamp(ping, 0).unwrap().with_timezone(&Local);
            let date_str = dt.format("%Y-%m-%d").to_string();
            days.entry(date_str).or_default().push(ping);
        }

        let mut result = Vec::new();
        for (date, day_pings) in days {
            let mut sessions = Vec::new();
            if day_pings.is_empty() {
                result.push(UsageDay { date, total_minutes: 0, sessions });
                continue;
            }

            let mut current_start = day_pings[0];
            let mut last_ping = day_pings[0];
            let mut total_seconds = 0;

            for i in 1..day_pings.len() {
                let gap = day_pings[i] - last_ping;
                if gap > threshold_seconds as i64 { 
                    let duration_secs = last_ping - current_start;
                    let duration_mins = ((duration_secs as f32 / 60.0).round() as u32).max(1);
                    sessions.push(UsageSession {
                        start: DateTime::from_timestamp(current_start, 0).unwrap().with_timezone(&Local).format("%H:%M").to_string(),
                        end: DateTime::from_timestamp(last_ping, 0).unwrap().with_timezone(&Local).format("%H:%M").to_string(),
                        start_ts: current_start,
                        end_ts: last_ping,
                        duration_minutes: duration_mins,
                    });
                    total_seconds += duration_secs;
                    current_start = day_pings[i];
                }
                last_ping = day_pings[i];
            }
            
            // Add the last session
            let duration_secs = last_ping - current_start;
            let duration_mins = ((duration_secs as f32 / 60.0).round() as u32).max(1);
            sessions.push(UsageSession {
                start: DateTime::from_timestamp(current_start, 0).unwrap().with_timezone(&Local).format("%H:%M").to_string(),
                end: DateTime::from_timestamp(last_ping, 0).unwrap().with_timezone(&Local).format("%H:%M").to_string(),
                start_ts: current_start,
                end_ts: last_ping,
                duration_minutes: duration_mins,
            });
            total_seconds += duration_secs;

            result.push(UsageDay {
                date,
                total_minutes: (total_seconds / 60) as u32,
                sessions,
            });
        }

        Ok(result)
    }
}
