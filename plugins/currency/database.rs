use rusqlite::{Connection, Result, params, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyAccount {
    pub user_id: String,
    pub username: String,
    pub balance: i64,
    pub lifetime_earned: i64,
    pub lifetime_spent: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn get_balance(conn: &Connection, channel: &str, username: &str) -> Result<i64> {
    conn.query_row(
        "SELECT coins FROM users WHERE channel = ?1 AND username = ?2",
        params![channel, username],
        |row| row.get(0),
    ).or(Ok(0))
}

pub fn get_account(conn: &Connection, user_id: &str) -> Result<Option<CurrencyAccount>> {
    conn.query_row(
        "SELECT user_id, username, balance, lifetime_earned, lifetime_spent, created_at, updated_at
         FROM currency_accounts WHERE user_id = ?1",
        params![user_id],
        |row| Ok(CurrencyAccount {
            user_id: row.get(0)?,
            username: row.get(1)?,
            balance: row.get(2)?,
            lifetime_earned: row.get(3)?,
            lifetime_spent: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        }),
    ).optional()
}

pub fn add_currency(
    conn: &Connection,
    channel: &str,
    username: &str,
    amount: i64,
    _reason: Option<&str>,
) -> Result<i64> {
    let now = current_timestamp();

    // Ensure account exists
    ensure_account(conn, channel, username)?;

    // Update balance
    conn.execute(
        "UPDATE users SET coins = coins + ?1, last_seen = ?2 WHERE channel = ?3 AND username = ?4",
        params![amount, now, channel, username],
    )?;

    // Get new balance
    get_balance(conn, channel, username)
}

pub fn deduct_currency(
    conn: &Connection,
    channel: &str,
    username: &str,
    amount: i64,
    _reason: Option<&str>,
) -> Result<i64> {
    let now = current_timestamp();

    // Check balance
    let balance = get_balance(conn, channel, username)?;
    if balance < amount {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    // Update balance
    conn.execute(
        "UPDATE users SET coins = coins - ?1, last_seen = ?2 WHERE channel = ?3 AND username = ?4",
        params![amount, now, channel, username],
    )?;

    // Get new balance
    get_balance(conn, channel, username)
}

pub fn transfer_currency(
    conn: &Connection,
    channel: &str,
    from_username: &str,
    to_username: &str,
    amount: i64,
) -> Result<()> {
    // Deduct from sender
    deduct_currency(conn, channel, from_username, amount, Some("Transfer"))?;

    // Add to receiver
    add_currency(conn, channel, to_username, amount, Some("Transfer received"))?;

    Ok(())
}

pub fn get_leaderboard(conn: &Connection, limit: usize) -> Result<Vec<CurrencyAccount>> {
    let mut stmt = conn.prepare(
        "SELECT user_id, username, balance, lifetime_earned, lifetime_spent, created_at, updated_at
         FROM currency_accounts
         ORDER BY balance DESC
         LIMIT ?1"
    )?;

    let accounts = stmt.query_map(params![limit], |row| {
        Ok(CurrencyAccount {
            user_id: row.get(0)?,
            username: row.get(1)?,
            balance: row.get(2)?,
            lifetime_earned: row.get(3)?,
            lifetime_spent: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(accounts)
}

fn ensure_account(conn: &Connection, channel: &str, username: &str) -> Result<()> {
    let now = current_timestamp();

    conn.execute(
        "INSERT OR IGNORE INTO users (channel, username, coins, spin_tokens, last_daily_spin, total_minutes, xp, level, total_messages, last_xp_gain, last_seen, created_at)
         VALUES (?1, ?2, 0, 0, 0, 0, 0, 1, 0, 0, ?3, ?3)",
        params![channel, username, now],
    )?;

    Ok(())
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
