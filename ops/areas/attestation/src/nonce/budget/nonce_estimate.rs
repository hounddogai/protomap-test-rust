use super::*;

use acme_kit::text;

/// Where a nonce estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl NonceEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for NonceEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceEstimate {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub cipher: String,
    pub batch_size: i64,
    pub minimum: f64,
    pub description: String,
    pub reviewer: String,
    pub status: NonceEstimateStatus,
}

impl NonceEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceEstimateStatus) -> bool {
        match self.status {
            NonceEstimateStatus::Open => {
                match next {
                    NonceEstimateStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceEstimateStatus::InReview => {
                return next == NonceEstimateStatus::Resolved || next == NonceEstimateStatus::Reopened;
            }
            NonceEstimateStatus::Resolved => {
                match next {
                    NonceEstimateStatus::Reopened | NonceEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceEstimateStatus::Reopened => {
                match next {
                    NonceEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.amount + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceEstimate {
        NonceEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            cipher: self.cipher.clone(),
            batch_size: self.batch_size,
            minimum: self.minimum,
            description: self.description.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a nonce estimate in status open.
pub fn new_nonce_estimate(code: &str, label: &str) -> NonceEstimate {
    NonceEstimate {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        cipher: "".to_string(),
        batch_size: 0,
        minimum: 0.0,
        description: "".to_string(),
        reviewer: "".to_string(),
        status: NonceEstimateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_nonce_estimate("code-713", "label-680");
        assert_eq!(item.status, NonceEstimateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_nonce_estimate("code-713", "label-680");
        if item.move_to(NonceEstimateStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceEstimateStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_nonce_estimate("code-713", "label-680");
        item.status = NonceEstimateStatus::Archived;
        assert!(
            item.move_to(NonceEstimateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
