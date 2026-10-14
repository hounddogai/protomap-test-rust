use super::*;

use acme_kit::text;

/// Where a grant digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GrantDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GrantDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant digest as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantDigest {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub grace_days: i64,
    pub quantity: i64,
    pub score: f64,
    pub maximum: f64,
    pub batch_size: i64,
    pub revision: i64,
    pub approved: bool,
    pub status: GrantDigestStatus,
}

impl GrantDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantDigestStatus) -> bool {
        match self.status {
            GrantDigestStatus::Proposed => {
                return next == GrantDigestStatus::Approved || next == GrantDigestStatus::Withdrawn;
            }
            GrantDigestStatus::Approved => {
                match next {
                    GrantDigestStatus::Rejected | GrantDigestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantDigestStatus::Rejected => {
                match next {
                    GrantDigestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: GrantDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.revision as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantDigest {
        GrantDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            grace_days: self.grace_days,
            quantity: self.quantity,
            score: self.score,
            maximum: self.maximum,
            batch_size: self.batch_size,
            revision: self.revision,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a grant digest in status proposed.
pub fn new_grant_digest(code: &str, title: &str) -> GrantDigest {
    GrantDigest {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        grace_days: 0,
        quantity: 0,
        score: 0.0,
        maximum: 0.0,
        batch_size: 0,
        revision: 0,
        approved: false,
        status: GrantDigestStatus::Proposed,
    }
}
