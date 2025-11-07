use rusqlite::{Connection, Result, params, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouletteGame {
    pub id: i64,
    pub channel: String,
    pub status: String,
    pub winning_number: Option<i64>,
    pub created_at: i64,
    pub spin_started_at: Option<i64>,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouletteBet {
    pub id: i64,
    pub game_id: i64,
    pub user_id: String,
    pub username: String,
    pub amount: i64,
    pub bet_type: String,
    pub bet_value: String,
    pub won: Option<bool>,
    pub payout: Option<i64>,
    pub created_at: i64,
}

pub fn start_game(conn: &Connection, channel: &str) -> Result<i64> {
    let now = current_timestamp();

    conn.execute(
        "INSERT INTO roulette_games (channel, status, created_at)
         VALUES (?1, 'betting', ?2)",
        params![channel, now],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn get_active_game(conn: &Connection, channel: &str) -> Result<Option<RouletteGame>> {
    conn.query_row(
        "SELECT id, channel, status, winning_number, created_at, spin_started_at, completed_at
         FROM roulette_games
         WHERE channel = ?1 AND status IN ('betting', 'spinning')
         ORDER BY created_at DESC
         LIMIT 1",
        params![channel],
        |row| Ok(RouletteGame {
            id: row.get(0)?,
            channel: row.get(1)?,
            status: row.get(2)?,
            winning_number: row.get(3)?,
            created_at: row.get(4)?,
            spin_started_at: row.get(5)?,
            completed_at: row.get(6)?,
        }),
    ).optional()
}

pub fn place_bet(
    conn: &Connection,
    channel: &str,
    user_id: &str,
    username: &str,
    amount: i64,
    bet_type: &str,
    bet_value: &str,
) -> Result<()> {
    let game = get_active_game(conn, channel)?;

    if let Some(game) = game {
        if game.status == "betting" {
            let now = current_timestamp();

            conn.execute(
                "INSERT INTO roulette_bets (game_id, user_id, username, amount, bet_type, bet_value, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![game.id, user_id, username, amount, bet_type, bet_value, now],
            )?;

            Ok(())
        } else {
            Err(rusqlite::Error::InvalidQuery)
        }
    } else {
        Err(rusqlite::Error::QueryReturnedNoRows)
    }
}

pub fn spin_wheel(conn: &Connection, channel: &str) -> Result<i64> {
    use rand::Rng;

    let game = get_active_game(conn, channel)?
        .ok_or(rusqlite::Error::QueryReturnedNoRows)?;

    if game.status != "betting" {
        return Err(rusqlite::Error::InvalidQuery);
    }

    // European roulette: 0-36
    let winning_number = rand::thread_rng().gen_range(0..=36);
    let now = current_timestamp();

    // Update game
    conn.execute(
        "UPDATE roulette_games SET status = 'ended', winning_number = ?1, spin_started_at = ?2, completed_at = ?2
         WHERE id = ?3",
        params![winning_number, now, game.id],
    )?;

    // Process bets
    process_bets(conn, game.id, winning_number)?;

    Ok(winning_number)
}

fn process_bets(conn: &Connection, game_id: i64, winning_number: i64) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, user_id, amount, bet_type, bet_value FROM roulette_bets WHERE game_id = ?1"
    )?;

    let bets: Vec<(i64, String, i64, String, String)> = stmt.query_map(params![game_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    })?
    .collect::<Result<Vec<_>>>()?;

    for (bet_id, _user_id, amount, bet_type, bet_value) in bets {
        let won = super::game::bet_wins(&bet_type, &bet_value, winning_number);
        let payout = if won {
            amount * (super::game::get_payout_multiplier(&bet_type) + 1)
        } else {
            0
        };

        conn.execute(
            "UPDATE roulette_bets SET won = ?1, payout = ?2 WHERE id = ?3",
            params![won, payout, bet_id],
        )?;
    }

    Ok(())
}

pub fn set_spin_result(conn: &Connection, game_id: i64, winning_number: i64) -> Result<()> {
    let now = current_timestamp();

    // Update game with result
    conn.execute(
        "UPDATE roulette_games SET status = 'ended', winning_number = ?1, completed_at = ?2
         WHERE id = ?3 AND status = 'spinning'",
        params![winning_number, now, game_id],
    )?;

    // Process bets
    process_bets(conn, game_id, winning_number)?;

    Ok(())
}

pub fn start_spin(conn: &Connection, game_id: i64) -> Result<()> {
    let now = current_timestamp();

    conn.execute(
        "UPDATE roulette_games SET status = 'spinning', spin_started_at = ?1 WHERE id = ?2",
        params![now, game_id],
    )?;

    Ok(())
}

pub fn get_game_bets(conn: &Connection, game_id: i64) -> Result<Vec<RouletteBet>> {
    let mut stmt = conn.prepare(
        "SELECT id, game_id, user_id, username, amount, bet_type, bet_value, won, payout, created_at
         FROM roulette_bets WHERE game_id = ?1"
    )?;

    let bets = stmt.query_map(params![game_id], |row| {
        Ok(RouletteBet {
            id: row.get(0)?,
            game_id: row.get(1)?,
            user_id: row.get(2)?,
            username: row.get(3)?,
            amount: row.get(4)?,
            bet_type: row.get(5)?,
            bet_value: row.get(6)?,
            won: row.get(7)?,
            payout: row.get(8)?,
            created_at: row.get(9)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(bets)
}

pub fn cancel_game(conn: &Connection, channel: &str) -> Result<()> {
    let game = get_active_game(conn, channel)?
        .ok_or(rusqlite::Error::QueryReturnedNoRows)?;

    let now = current_timestamp();

    conn.execute(
        "UPDATE roulette_games SET status = 'cancelled', completed_at = ?1 WHERE id = ?2",
        params![now, game.id],
    )?;

    Ok(())
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
