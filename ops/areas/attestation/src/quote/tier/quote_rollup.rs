use super::*;

use acme_kit::text;

/// Where a quote rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl QuoteRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for QuoteRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote rollup as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRollup {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub approved: bool,
    pub discount: f64,
    pub sequence: i64,
    pub created_at: i64,
    pub currency: String,
    pub threshold: f64,
    pub status: QuoteRollupStatus,
}

impl QuoteRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteRollupStatus) -> bool {
        match self.status {
            QuoteRollupStatus::Requested => {
                return next == QuoteRollupStatus::Granted;
            }
            QuoteRollupStatus::Granted => {
                match next {
                    QuoteRollupStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteRollupStatus::Revoked => {
                return next == QuoteRollupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "quote rollup", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.discount - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteRollup {
        QuoteRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            approved: self.approved,
            discount: self.discount,
            sequence: self.sequence,
            created_at: self.created_at,
            currency: self.currency.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a quote rollup in status requested.
pub fn new_quote_rollup(code: &str, label: &str) -> QuoteRollup {
    QuoteRollup {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        approved: false,
        discount: 0.0,
        sequence: 0,
        created_at: 0,
        currency: "".to_string(),
        threshold: 0.0,
        status: QuoteRollupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_quote_rollup("code-657", "label-898");
        assert_eq!(item.status, QuoteRollupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_quote_rollup("code-657", "label-898");
        if item.move_to(QuoteRollupStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteRollupStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_quote_rollup("code-657", "label-898");
        item.status = QuoteRollupStatus::Expired;
        assert!(
            item.move_to(QuoteRollupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
