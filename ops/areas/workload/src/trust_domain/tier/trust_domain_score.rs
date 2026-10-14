use super::*;

use acme_kit::text;

/// Where a trust domain score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TrustDomainScoreStatus {
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

impl std::fmt::Display for TrustDomainScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a trust domain score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScore {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub ttl_seconds: i64,
    pub quantity: i64,
    pub pinned: bool,
    pub status: TrustDomainScoreStatus,
}

impl TrustDomainScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainScoreStatus) -> bool {
        match self.status {
            TrustDomainScoreStatus::Open => {
                match next {
                    TrustDomainScoreStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainScoreStatus::InReview => {
                match next {
                    TrustDomainScoreStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainScoreStatus::Resolved => {
                match next {
                    TrustDomainScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainScoreStatus::Reopened => {
                return next == TrustDomainScoreStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain score {} cannot move from {} to {}",
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
        ((self.maximum - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainScore {
        TrustDomainScore {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            ttl_seconds: self.ttl_seconds,
            quantity: self.quantity,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a trust domain score in status open.
pub fn new_trust_domain_score(code: &str, label: &str) -> TrustDomainScore {
    TrustDomainScore {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        ttl_seconds: 0,
        quantity: 0,
        pinned: false,
        status: TrustDomainScoreStatus::Open,
    }
}
