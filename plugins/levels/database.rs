use rusqlite::{Connection, Result, params, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLevel {
    pub id: i64,
    pub channel: String,
    pub username: String,
    pub xp: i64,
    pub level: i64,
    pub total_messages: i64,
    pub last_xp_gain: i64,
}

pub fn calculate_level(xp: i64) -> i64 {
    // Level formula: level = floor(sqrt(xp / 100))
    ((xp as f64 / 100.0).sqrt().floor() as i64).max(1)
}

pub fn xp_for_level(level: i64) -> i64 {
    // Reverse: xp = level^2 * 100
    level * level * 100
}

pub fn add_xp(
    conn: &Connection,
    channel: &str,
    username: &str,
    amount: i64,
    reason: Option<&str>,
) -> Result<(i64, i64)> {
    let now = current_timestamp();

    // Get current level or create user
    let old_level = if let Some(user_level) = get_user_level(conn, channel, username)? {
        user_level.level
    } else {
        // Create new user
        conn.execute(
            "INSERT INTO user_levels (channel, username, xp, level, total_messages, last_xp_gain)
             VALUES (?1, ?2, 0, 1, 0, ?3)",
            params![channel, username, now],
        )?;
        1
    };

    // Add XP
    conn.execute(
        "UPDATE user_levels SET xp = xp + ?1, last_xp_gain = ?2 WHERE channel = ?3 AND username = ?4",
        params![amount, now, channel, username],
    )?;

    // Record transaction (using channel:username as user_id for compatibility)
    let user_id = format!("{}:{}", channel, username);
    conn.execute(
        "INSERT INTO xp_transactions (user_id, amount, reason, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![user_id, amount, reason, now],
    )?;

    // Get new total XP and calculate level
    let total_xp: i64 = conn.query_row(
        "SELECT xp FROM user_levels WHERE channel = ?1 AND username = ?2",
        params![channel, username],
        |row| row.get(0),
    )?;

    let new_level = calculate_level(total_xp);

    // Update level if changed
    if new_level != old_level {
        conn.execute(
            "UPDATE user_levels SET level = ?1 WHERE channel = ?2 AND username = ?3",
            params![new_level, channel, username],
        )?;
    }

    Ok((old_level, new_level))
}

pub fn get_user_level(conn: &Connection, channel: &str, username: &str) -> Result<Option<UserLevel>> {
    conn.query_row(
        "SELECT id, channel, username, xp, level, total_messages, last_xp_gain
         FROM user_levels WHERE channel = ?1 AND username = ?2",
        params![channel, username],
        |row| Ok(UserLevel {
            id: row.get(0)?,
            channel: row.get(1)?,
            username: row.get(2)?,
            xp: row.get(3)?,
            level: row.get(4)?,
            total_messages: row.get(5)?,
            last_xp_gain: row.get(6)?,
        }),
    ).optional()
}

pub fn get_leaderboard(conn: &Connection, channel: &str, limit: usize) -> Result<Vec<UserLevel>> {
    let mut stmt = conn.prepare(
        "SELECT id, channel, username, xp, level, total_messages, last_xp_gain
         FROM user_levels
         WHERE channel = ?1
         ORDER BY xp DESC
         LIMIT ?2"
    )?;

    let users = stmt.query_map(params![channel, limit], |row| {
        Ok(UserLevel {
            id: row.get(0)?,
            channel: row.get(1)?,
            username: row.get(2)?,
            xp: row.get(3)?,
            level: row.get(4)?,
            total_messages: row.get(5)?,
            last_xp_gain: row.get(6)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(users)
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
