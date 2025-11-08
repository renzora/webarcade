// Auction plugin database operations
use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use crate::core::plugin_context::PluginContext;
use super::events::{AuctionEndedEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auction {
    pub id: i64,
    pub item_id: String,
    pub item_name: String,
    pub item_rarity: String,
    pub starting_bid: i64,
    pub current_bid: Option<i64>,
    pub current_bidder: Option<String>,
    pub created_by: String,
    pub status: String,
    pub created_at: i64,
    pub ends_at: i64,
    pub ended_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionBid {
    pub id: i64,
    pub auction_id: i64,
    pub user_id: String,
    pub username: String,
    pub amount: i64,
    pub created_at: i64,
}

pub fn create_auction(
    conn: &Connection,
    item_id: &str,
    item_name: &str,
    item_rarity: &str,
    starting_bid: i64,
    created_by: &str,
    duration_seconds: i64,
) -> Result<i64> {
    let now = current_timestamp();
    let ends_at = now + duration_seconds;

    conn.execute(
        "INSERT INTO auctions (item_id, item_name, item_rarity, starting_bid, created_by, created_at, ends_at, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'active')",
        params![item_id, item_name, item_rarity, starting_bid, created_by, now, ends_at],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn place_bid(
    conn: &Connection,
    auction_id: i64,
    user_id: &str,
    username: &str,
    amount: i64,
) -> Result<()> {
    // Check if auction exists and is active
    let status: String = conn.query_row(
        "SELECT status FROM auctions WHERE id = ?1",
        params![auction_id],
        |row| row.get(0),
    )?;

    if status != "active" {
        return Err(rusqlite::Error::InvalidQuery);
    }

    // Check if bid is higher than current bid
    let current_bid: Option<i64> = conn.query_row(
        "SELECT current_bid FROM auctions WHERE id = ?1",
        params![auction_id],
        |row| row.get(0),
    ).ok();

    if let Some(current) = current_bid {
        if amount <= current {
            return Err(rusqlite::Error::InvalidQuery);
        }
    }

    // Insert bid
    conn.execute(
        "INSERT INTO auction_bids (auction_id, user_id, username, amount, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![auction_id, user_id, username, amount, current_timestamp()],
    )?;

    // Update current bid
    conn.execute(
        "UPDATE auctions SET current_bid = ?1, current_bidder = ?2 WHERE id = ?3",
        params![amount, username, auction_id],
    )?;

    Ok(())
}

pub fn get_active_auctions(conn: &Connection) -> Result<Vec<Auction>> {
    let mut stmt = conn.prepare(
        "SELECT id, item_id, item_name, item_rarity, starting_bid, current_bid,
                current_bidder, created_by, status, created_at, ends_at, ended_at
         FROM auctions WHERE status = 'active' ORDER BY ends_at ASC"
    )?;

    let auctions = stmt.query_map([], |row| {
        Ok(Auction {
            id: row.get(0)?,
            item_id: row.get(1)?,
            item_name: row.get(2)?,
            item_rarity: row.get(3)?,
            starting_bid: row.get(4)?,
            current_bid: row.get(5)?,
            current_bidder: row.get(6)?,
            created_by: row.get(7)?,
            status: row.get(8)?,
            created_at: row.get(9)?,
            ends_at: row.get(10)?,
            ended_at: row.get(11)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(auctions)
}

pub fn check_ended_auctions(conn: &Connection, ctx: &PluginContext) -> Result<()> {
    let now = current_timestamp();

    // Find auctions that have ended
    let mut stmt = conn.prepare(
        "SELECT id, current_bidder, current_bid FROM auctions
         WHERE status = 'active' AND ends_at <= ?1"
    )?;

    let ended_auctions: Vec<(i64, Option<String>, Option<i64>)> = stmt.query_map([now], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?
    .collect::<Result<Vec<_>>>()?;

    // Mark them as ended and emit events
    for (auction_id, winner, final_bid) in ended_auctions {
        conn.execute(
            "UPDATE auctions SET status = 'ended', ended_at = ?1 WHERE id = ?2",
            params![now, auction_id],
        )?;

        // Emit event
        ctx.emit("auction.ended", &AuctionEndedEvent {
            auction_id,
            winner: winner.clone(),
            final_bid,
        });

        log::info!("[Auction] Auction {} ended. Winner: {:?}, Bid: {:?}",
                   auction_id, winner, final_bid);
    }

    Ok(())
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
