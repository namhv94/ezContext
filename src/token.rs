use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenSavings {
    pub original_tokens_est: usize,
    pub skeleton_tokens_est: usize,
    pub tokens_saved: usize,
    pub percentage_saved: f64,
    pub estimated_cost_saved_usd: f64,
}

pub struct TokenEstimator;

impl TokenEstimator {
    /// Estimates token count using standard BPE heuristic (~4 characters per token).
    pub fn estimate_tokens(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }
        let char_count = text.chars().count();
        char_count.div_ceil(4)
    }

    /// Compares original code and extracted skeleton to calculate token & cost savings.
    /// Default pricing: $3.00 per 1M input tokens (Claude 3.5 Sonnet tier).
    pub fn calculate_savings(original: &str, skeleton: &str) -> TokenSavings {
        let orig_tokens = Self::estimate_tokens(original);
        let skel_tokens = Self::estimate_tokens(skeleton);
        let saved = orig_tokens.saturating_sub(skel_tokens);

        let pct = if orig_tokens > 0 {
            (saved as f64 / orig_tokens as f64) * 100.0
        } else {
            0.0
        };

        // $3.00 per 1M input tokens
        let cost_saved = (saved as f64 / 1_000_000.0) * 3.0;

        TokenSavings {
            original_tokens_est: orig_tokens,
            skeleton_tokens_est: skel_tokens,
            tokens_saved: saved,
            percentage_saved: pct,
            estimated_cost_saved_usd: cost_saved,
        }
    }
}
