use super::*;

use acme_kit::text;

/// Where a rule tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RuleTierStatus {
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

impl std::fmt::Display for RuleTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleTier {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub weight: f64,
    pub cluster: String,
    pub issuer: String,
    pub retries: i64,
    pub key_id: String,
    pub ratio: f64,
    pub sequence: i64,
    pub status: RuleTierStatus,
}

impl RuleTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleTierStatus) -> bool {
        match self.status {
            RuleTierStatus::Open => {
                match next {
                    RuleTierStatus::Resolved | RuleTierStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleTierStatus::InReview => {
                return next == RuleTierStatus::Resolved;
            }
            RuleTierStatus::Resolved => {
                match next {
                    RuleTierStatus::Reopened | RuleTierStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleTierStatus::Reopened => {
                return next == RuleTierStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rule tier", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.retries as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleTier {
        RuleTier {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            weight: self.weight,
            cluster: self.cluster.clone(),
            issuer: self.issuer.clone(),
            retries: self.retries,
            key_id: self.key_id.clone(),
            ratio: self.ratio,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rule tier in status open.
pub fn new_rule_tier(code: &str, label: &str) -> RuleTier {
    RuleTier {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        weight: 0.0,
        cluster: "".to_string(),
        issuer: "".to_string(),
        retries: 0,
        key_id: "".to_string(),
        ratio: 0.0,
        sequence: 0,
        status: RuleTierStatus::Open,
    }
}
