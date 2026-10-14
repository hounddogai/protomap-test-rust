use super::*;

use acme_kit::text;

/// Where a ttl policy window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TtlPolicyWindowStatus {
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

impl std::fmt::Display for TtlPolicyWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a ttl policy window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyWindow {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub currency: String,
    pub maximum: f64,
    pub phone: String,
    pub priority: i64,
    pub region: String,
    pub cipher: String,
    pub floor_value: f64,
    pub status: TtlPolicyWindowStatus,
}

impl TtlPolicyWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyWindowStatus) -> bool {
        match self.status {
            TtlPolicyWindowStatus::Open => {
                match next {
                    TtlPolicyWindowStatus::Reopened | TtlPolicyWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyWindowStatus::InReview => {
                match next {
                    TtlPolicyWindowStatus::Reopened | TtlPolicyWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyWindowStatus::Resolved => {
                return next == TtlPolicyWindowStatus::Reopened || next == TtlPolicyWindowStatus::Archived;
            }
            TtlPolicyWindowStatus::Reopened => {
                match next {
                    TtlPolicyWindowStatus::Archived => {
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

    pub fn move_to(&mut self, next: TtlPolicyWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.retries as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyWindow {
        TtlPolicyWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            currency: self.currency.clone(),
            maximum: self.maximum,
            phone: self.phone.clone(),
            priority: self.priority,
            region: self.region.clone(),
            cipher: self.cipher.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a ttl policy window in status open.
pub fn new_ttl_policy_window(code: &str, label: &str) -> TtlPolicyWindow {
    TtlPolicyWindow {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        currency: "".to_string(),
        maximum: 0.0,
        phone: "".to_string(),
        priority: 0,
        region: "".to_string(),
        cipher: "".to_string(),
        floor_value: 0.0,
        status: TtlPolicyWindowStatus::Open,
    }
}
