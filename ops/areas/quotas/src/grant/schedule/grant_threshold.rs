use super::*;

use acme_kit::text;

/// Where a grant threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantThresholdStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GrantThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GrantThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant threshold as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantThreshold {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub burst: i64,
    pub contact_email: String,
    pub category: String,
    pub score: f64,
    pub status: GrantThresholdStatus,
}

impl GrantThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantThresholdStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantThresholdStatus) -> bool {
        match self.status {
            GrantThresholdStatus::Requested => {
                match next {
                    GrantThresholdStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantThresholdStatus::Granted => {
                return next == GrantThresholdStatus::Expired;
            }
            GrantThresholdStatus::Revoked => {
                return next == GrantThresholdStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "grant threshold", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.window_days as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantThreshold {
        GrantThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            burst: self.burst,
            contact_email: self.contact_email.clone(),
            category: self.category.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a grant threshold in status requested.
pub fn new_grant_threshold(code: &str, name: &str) -> GrantThreshold {
    GrantThreshold {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        burst: 0,
        contact_email: "".to_string(),
        category: "".to_string(),
        score: 0.0,
        status: GrantThresholdStatus::Requested,
    }
}
