use crate::domain::MarketSide;

/// Logarithmic Market Scoring Rule (LMSR) - Polymarket style
///
/// LMSR is a market maker designed for prediction markets that:
/// - Never runs out of liquidity (shares are minted/burned on demand)
/// - Uses a liquidity parameter `b` to control market depth
/// - Provides better pricing for prediction markets
/// - YES + NO shares always sum to $1 in value
///
/// Cost function: C(q) = b * ln(e^(q_yes/b) + e^(q_no/b))
/// where q_yes and q_no are outstanding shares
///
/// The liquidity parameter `b` controls market depth:
/// - Higher `b` = more liquidity, less price movement per trade
/// - Lower `b` = less liquidity, more price movement per trade
pub struct LmsrPricing;

impl LmsrPricing {
    /// Calculate the LMSR cost function
    /// C(q) = b * ln(e^(q_yes/b) + e^(q_no/b))
    fn cost_function(q_yes: f64, q_no: f64, b: f64) -> f64 {
        let exp_yes = (q_yes / b).exp();
        let exp_no = (q_no / b).exp();
        b * (exp_yes + exp_no).ln()
    }

    /// Calculate the cost to buy shares
    ///
    /// # Arguments
    /// * `q_yes` - Current outstanding YES shares
    /// * `q_no` - Current outstanding NO shares
    /// * `shares` - Number of shares to buy
    /// * `side` - Which side (YES or NO) to buy
    /// * `b` - Liquidity parameter (higher = more liquid market)
    ///
    /// # Returns
    /// Cost in currency to buy the shares
    pub fn calculate_buy_cost(
        q_yes: f64,
        q_no: f64,
        shares: f64,
        side: MarketSide,
        b: f64,
    ) -> Result<f64, String> {
        if shares <= 0.0 {
            return Err("Shares must be positive".to_string());
        }
        if b <= 0.0 {
            return Err("Liquidity parameter must be positive".to_string());
        }

        let cost_before = Self::cost_function(q_yes, q_no, b);

        let cost_after = match side {
            MarketSide::Yes => {
                Self::cost_function(q_yes + shares, q_no, b)
            }
            MarketSide::No => {
                Self::cost_function(q_yes, q_no + shares, b)
            }
        };

        let cost = cost_after - cost_before;

        if cost < 0.0 {
            return Err("Invalid calculation resulted in negative cost".to_string());
        }

        Ok(cost)
    }

    /// Calculate proceeds from selling shares
    ///
    /// # Arguments
    /// * `q_yes` - Current outstanding YES shares
    /// * `q_no` - Current outstanding NO shares
    /// * `shares` - Number of shares to sell
    /// * `side` - Which side (YES or NO) to sell
    /// * `b` - Liquidity parameter
    ///
    /// # Returns
    /// Amount received in currency for selling the shares
    pub fn calculate_sell_proceeds(
        q_yes: f64,
        q_no: f64,
        shares: f64,
        side: MarketSide,
        b: f64,
    ) -> Result<f64, String> {
        if shares <= 0.0 {
            return Err("Shares must be positive".to_string());
        }
        if b <= 0.0 {
            return Err("Liquidity parameter must be positive".to_string());
        }

        // Check if user has enough shares
        let current_shares = match side {
            MarketSide::Yes => q_yes,
            MarketSide::No => q_no,
        };

        if shares > current_shares {
            return Err("Not enough shares to sell".to_string());
        }

        let cost_before = Self::cost_function(q_yes, q_no, b);

        let cost_after = match side {
            MarketSide::Yes => {
                Self::cost_function(q_yes - shares, q_no, b)
            }
            MarketSide::No => {
                Self::cost_function(q_yes, q_no - shares, b)
            }
        };

        let proceeds = cost_before - cost_after;

        if proceeds < 0.0 {
            return Err("Invalid calculation resulted in negative proceeds".to_string());
        }

        Ok(proceeds)
    }

    /// Calculate the current implied probability of YES
    ///
    /// Probability = e^(q_yes/b) / (e^(q_yes/b) + e^(q_no/b))
    pub fn implied_probability(q_yes: f64, q_no: f64, b: f64) -> f64 {
        let exp_yes = (q_yes / b).exp();
        let exp_no = (q_no / b).exp();
        exp_yes / (exp_yes + exp_no)
    }

    /// Calculate the instantaneous price for the next marginal share
    /// This is the derivative of the cost function
    pub fn instantaneous_price(q_yes: f64, q_no: f64, side: MarketSide, b: f64) -> f64 {
        Self::implied_probability(q_yes, q_no, b) * match side {
            MarketSide::Yes => 1.0,
            MarketSide::No => 0.0,
        } + (1.0 - Self::implied_probability(q_yes, q_no, b)) * match side {
            MarketSide::Yes => 0.0,
            MarketSide::No => 1.0,
        }
    }
}

// Keep old AmmPricing for backward compatibility during migration
pub struct AmmPricing;

impl AmmPricing {
    /// Calculate the cost to buy a given number of shares
    ///
    /// # Arguments
    /// * `yes_pool` - Current liquidity in the YES pool
    /// * `no_pool` - Current liquidity in the NO pool
    /// * `shares` - Number of shares to buy
    /// * `side` - Which side (YES or NO) to buy
    ///
    /// # Returns
    /// Cost in currency to buy the shares
    pub fn calculate_buy_cost(
        yes_pool: f64,
        no_pool: f64,
        shares: f64,
        side: MarketSide,
    ) -> Result<f64, String> {
        if shares <= 0.0 {
            return Err("Shares must be positive".to_string());
        }
        if yes_pool <= 0.0 || no_pool <= 0.0 {
            return Err("Pools must be positive".to_string());
        }

        let k = yes_pool * no_pool;

        let cost = match side {
            MarketSide::Yes => {
                // Buying YES means removing from yes_pool
                let new_yes_pool = yes_pool - shares;
                if new_yes_pool <= 0.0 {
                    return Err("Not enough liquidity in YES pool".to_string());
                }
                let new_no_pool = k / new_yes_pool;
                new_no_pool - no_pool
            }
            MarketSide::No => {
                // Buying NO means removing from no_pool
                let new_no_pool = no_pool - shares;
                if new_no_pool <= 0.0 {
                    return Err("Not enough liquidity in NO pool".to_string());
                }
                let new_yes_pool = k / new_no_pool;
                new_yes_pool - yes_pool
            }
        };

        if cost < 0.0 {
            return Err("Invalid calculation resulted in negative cost".to_string());
        }

        Ok(cost)
    }

    /// Calculate how much you receive for selling shares
    ///
    /// # Arguments
    /// * `yes_pool` - Current liquidity in the YES pool
    /// * `no_pool` - Current liquidity in the NO pool
    /// * `shares` - Number of shares to sell
    /// * `side` - Which side (YES or NO) to sell
    ///
    /// # Returns
    /// Amount received in currency for selling the shares
    pub fn calculate_sell_proceeds(
        yes_pool: f64,
        no_pool: f64,
        shares: f64,
        side: MarketSide,
    ) -> Result<f64, String> {
        if shares <= 0.0 {
            return Err("Shares must be positive".to_string());
        }
        if yes_pool <= 0.0 || no_pool <= 0.0 {
            return Err("Pools must be positive".to_string());
        }

        let k = yes_pool * no_pool;

        let proceeds = match side {
            MarketSide::Yes => {
                // Selling YES means adding back to yes_pool
                let new_yes_pool = yes_pool + shares;
                let new_no_pool = k / new_yes_pool;
                no_pool - new_no_pool
            }
            MarketSide::No => {
                // Selling NO means adding back to no_pool
                let new_no_pool = no_pool + shares;
                let new_yes_pool = k / new_no_pool;
                yes_pool - new_yes_pool
            }
        };

        if proceeds < 0.0 {
            return Err("Invalid calculation resulted in negative proceeds".to_string());
        }

        Ok(proceeds)
    }

    /// Calculate the current implied probability of YES
    ///
    /// Probability = no_pool / (yes_pool + no_pool)
    pub fn implied_probability(yes_pool: f64, no_pool: f64) -> f64 {
        let total = yes_pool + no_pool;
        if total == 0.0 {
            return 0.5; // Default to 50% if pools are empty
        }
        no_pool / total
    }

    /// Calculate the current price for a marginal share
    pub fn current_price(yes_pool: f64, no_pool: f64, side: MarketSide) -> f64 {
        Self::implied_probability(yes_pool, no_pool) * match side {
            MarketSide::Yes => 1.0,
            MarketSide::No => 0.0,
        } + (1.0 - Self::implied_probability(yes_pool, no_pool)) * match side {
            MarketSide::Yes => 0.0,
            MarketSide::No => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // LMSR Tests
    #[test]
    fn test_lmsr_initial_probability() {
        // Equal shares should give 50% probability
        let b = 100.0;
        let prob = LmsrPricing::implied_probability(0.0, 0.0, b);
        assert!((prob - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_lmsr_buy_increases_probability() {
        let b = 100.0;
        let q_yes = 0.0;
        let q_no = 0.0;

        let initial_prob = LmsrPricing::implied_probability(q_yes, q_no, b);

        // Buy 10 YES shares
        let cost = LmsrPricing::calculate_buy_cost(q_yes, q_no, 10.0, MarketSide::Yes, b).unwrap();
        assert!(cost > 0.0);

        let new_q_yes = q_yes + 10.0;
        let new_prob = LmsrPricing::implied_probability(new_q_yes, q_no, b);

        // Probability of YES should increase
        assert!(new_prob > initial_prob);
    }

    #[test]
    fn test_lmsr_no_liquidity_limit() {
        let b = 100.0;
        let q_yes = 0.0;
        let q_no = 0.0;

        // Should be able to buy large amounts (unlike CPMM)
        let cost = LmsrPricing::calculate_buy_cost(q_yes, q_no, 1000.0, MarketSide::Yes, b);
        assert!(cost.is_ok());
        assert!(cost.unwrap() > 0.0);
    }

    #[test]
    fn test_lmsr_buy_and_sell() {
        let b = 100.0;
        let q_yes = 10.0;
        let q_no = 5.0;

        // Buy 10 YES shares
        let buy_cost = LmsrPricing::calculate_buy_cost(q_yes, q_no, 10.0, MarketSide::Yes, b).unwrap();
        let new_q_yes = q_yes + 10.0;

        // Sell 10 YES shares
        let sell_proceeds = LmsrPricing::calculate_sell_proceeds(new_q_yes, q_no, 10.0, MarketSide::Yes, b).unwrap();

        // In LMSR, buy and immediate sell should get you back approximately the same amount
        // (within floating point error)
        assert!((sell_proceeds - buy_cost).abs() < 0.01);
        assert!(sell_proceeds > 0.0);
    }

    #[test]
    fn test_lmsr_price_bounds() {
        let b = 100.0;

        // Very high YES shares should give probability near 1
        let prob_high = LmsrPricing::implied_probability(1000.0, 0.0, b);
        assert!(prob_high > 0.99);

        // Very high NO shares should give probability near 0
        let prob_low = LmsrPricing::implied_probability(0.0, 1000.0, b);
        assert!(prob_low < 0.01);
    }

    #[test]
    fn test_lmsr_invalid_inputs() {
        let b = 100.0;
        assert!(LmsrPricing::calculate_buy_cost(0.0, 0.0, -10.0, MarketSide::Yes, b).is_err());
        assert!(LmsrPricing::calculate_buy_cost(0.0, 0.0, 10.0, MarketSide::Yes, 0.0).is_err());
        assert!(LmsrPricing::calculate_buy_cost(0.0, 0.0, 10.0, MarketSide::Yes, -10.0).is_err());
    }

    // Old CPMM Tests (kept for backward compatibility)
    #[test]
    fn test_initial_probability() {
        // Equal pools should give 50% probability
        let prob = AmmPricing::implied_probability(100.0, 100.0);
        assert!((prob - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_buy_cost_increases_probability() {
        let yes_pool = 100.0;
        let no_pool = 100.0;
        let initial_prob = AmmPricing::implied_probability(yes_pool, no_pool);

        // Buy 10 YES shares
        let cost = AmmPricing::calculate_buy_cost(yes_pool, no_pool, 10.0, MarketSide::Yes).unwrap();

        let new_yes_pool = yes_pool - 10.0;
        let new_no_pool = no_pool + cost;
        let new_prob = AmmPricing::implied_probability(new_yes_pool, new_no_pool);

        // Probability of YES should increase
        assert!(new_prob > initial_prob);
    }

    #[test]
    fn test_constant_product() {
        let yes_pool = 100.0;
        let no_pool = 100.0;
        let k = yes_pool * no_pool;

        let cost = AmmPricing::calculate_buy_cost(yes_pool, no_pool, 10.0, MarketSide::Yes).unwrap();

        let new_yes_pool = yes_pool - 10.0;
        let new_no_pool = no_pool + cost;
        let new_k = new_yes_pool * new_no_pool;

        // k should remain approximately constant (allowing for floating point errors)
        assert!((new_k - k).abs() < 0.01);
    }

    #[test]
    fn test_buy_and_sell_roundtrip() {
        let yes_pool = 100.0;
        let no_pool = 100.0;

        // Buy 10 YES shares
        let buy_cost = AmmPricing::calculate_buy_cost(yes_pool, no_pool, 10.0, MarketSide::Yes).unwrap();
        let new_yes_pool = yes_pool - 10.0;
        let new_no_pool = no_pool + buy_cost;

        // Sell 10 YES shares
        let sell_proceeds = AmmPricing::calculate_sell_proceeds(new_yes_pool, new_no_pool, 10.0, MarketSide::Yes).unwrap();

        // You should get back approximately the same (within floating point error)
        assert!((sell_proceeds - buy_cost).abs() < 0.01);
    }

    #[test]
    fn test_invalid_inputs() {
        assert!(AmmPricing::calculate_buy_cost(100.0, 100.0, -10.0, MarketSide::Yes).is_err());
        assert!(AmmPricing::calculate_buy_cost(0.0, 100.0, 10.0, MarketSide::Yes).is_err());
        assert!(AmmPricing::calculate_buy_cost(100.0, 100.0, 150.0, MarketSide::Yes).is_err());
    }
}
