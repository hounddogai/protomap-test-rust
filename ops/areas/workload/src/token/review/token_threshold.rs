use super::*;

use acme_kit::text;

/// Where a token threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenThresholdStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TokenThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TokenThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a token threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenThreshold {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub checksum: String,
    pub duration_minutes: i64,
    pub score: f64,
    pub floor_value: f64,
    pub owner_id: String,
    pub status: TokenThresholdStatus,
}

impl TokenThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenThresholdStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenThresholdStatus) -> bool {
        match self.status {
            TokenThresholdStatus::Proposed => {
                return next == TokenThresholdStatus::Rejected;
            }
            TokenThresholdStatus::Approved => {
                match next {
                    TokenThresholdStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenThresholdStatus::Rejected => {
                return next == TokenThresholdStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.threshold + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenThreshold {
        TokenThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            checksum: self.checksum.clone(),
            duration_minutes: self.duration_minutes,
            score: self.score,
            floor_value: self.floor_value,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a token threshold in status proposed.
pub fn new_token_threshold(code: &str, label: &str) -> TokenThreshold {
    TokenThreshold {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        checksum: "".to_string(),
        duration_minutes: 0,
        score: 0.0,
        floor_value: 0.0,
        owner_id: "".to_string(),
        status: TokenThresholdStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_token_threshold("code-706", "label-905");
        assert_eq!(item.status, TokenThresholdStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_token_threshold("code-706", "label-905");
        if item.move_to(TokenThresholdStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenThresholdStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_token_threshold("code-706", "label-905");
        item.status = TokenThresholdStatus::Withdrawn;
        assert!(
            item.move_to(TokenThresholdStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
