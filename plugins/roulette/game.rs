// Roulette game logic (European roulette)

/// Get the color of a roulette number
pub fn get_number_color(num: i64) -> &'static str {
    match num {
        0 => "green",
        1 | 3 | 5 | 7 | 9 | 12 | 14 | 16 | 18 | 19 | 21 | 23 | 25 | 27 | 30 | 32 | 34 | 36 => "red",
        _ => "black"
    }
}

/// Calculate payout multiplier for a bet type
pub fn get_payout_multiplier(bet_type: &str) -> i64 {
    match bet_type {
        "number" => 35,  // 35:1
        "red" | "black" | "odd" | "even" | "low" | "high" => 1,  // 1:1
        "dozen1" | "dozen2" | "dozen3" | "column1" | "column2" | "column3" => 2,  // 2:1
        _ => 0
    }
}

/// Check if a number wins for a bet
pub fn bet_wins(bet_type: &str, bet_value: &str, winning_number: i64) -> bool {
    match bet_type {
        "number" => {
            if let Ok(num) = bet_value.parse::<i64>() {
                num == winning_number
            } else {
                false
            }
        },
        "red" => get_number_color(winning_number) == "red",
        "black" => get_number_color(winning_number) == "black",
        "odd" => winning_number > 0 && winning_number % 2 == 1,
        "even" => winning_number > 0 && winning_number % 2 == 0,
        "low" => winning_number >= 1 && winning_number <= 18,
        "high" => winning_number >= 19 && winning_number <= 36,
        "dozen1" => winning_number >= 1 && winning_number <= 12,
        "dozen2" => winning_number >= 13 && winning_number <= 24,
        "dozen3" => winning_number >= 25 && winning_number <= 36,
        "column1" => winning_number > 0 && winning_number % 3 == 1,
        "column2" => winning_number > 0 && winning_number % 3 == 2,
        "column3" => winning_number > 0 && winning_number % 3 == 0,
        _ => false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colors() {
        assert_eq!(get_number_color(0), "green");
        assert_eq!(get_number_color(1), "red");
        assert_eq!(get_number_color(2), "black");
        assert_eq!(get_number_color(18), "red");
        assert_eq!(get_number_color(19), "red");
    }

    #[test]
    fn test_bet_wins() {
        // Number bet
        assert!(bet_wins("number", "5", 5));
        assert!(!bet_wins("number", "5", 6));

        // Color bets
        assert!(bet_wins("red", "", 1));
        assert!(!bet_wins("red", "", 2));
        assert!(bet_wins("black", "", 2));

        // Odd/even
        assert!(bet_wins("odd", "", 1));
        assert!(!bet_wins("odd", "", 2));
        assert!(bet_wins("even", "", 2));

        // Low/high
        assert!(bet_wins("low", "", 18));
        assert!(!bet_wins("low", "", 19));
        assert!(bet_wins("high", "", 19));

        // Dozens
        assert!(bet_wins("dozen1", "", 12));
        assert!(!bet_wins("dozen1", "", 13));
        assert!(bet_wins("dozen2", "", 13));
    }
}
