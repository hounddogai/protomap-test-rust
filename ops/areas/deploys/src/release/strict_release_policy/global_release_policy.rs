use super::*;

use acme_kit::text;

/// Where a global release policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReleasePolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalReleasePolicyStatus {
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

impl std::fmt::Display for GlobalReleasePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global release policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleasePolicy {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub key_id: String,
    pub minimum: f64,
    pub rate: f64,
    pub host: String,
    pub attempts: i64,
    pub region: String,
    pub channel: String,
    pub issuer: String,
    pub status: GlobalReleasePolicyStatus,
}

impl GlobalReleasePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReleasePolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReleasePolicyStatus) -> bool {
        match self.status {
            GlobalReleasePolicyStatus::Open => {
                match next {
                    GlobalReleasePolicyStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleasePolicyStatus::InReview => {
                return next == GlobalReleasePolicyStatus::Resolved;
            }
            GlobalReleasePolicyStatus::Resolved => {
                return next == GlobalReleasePolicyStatus::Archived;
            }
            GlobalReleasePolicyStatus::Reopened => {
                match next {
                    GlobalReleasePolicyStatus::Archived => {
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

    pub fn move_to(&mut self, next: GlobalReleasePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global release policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.attempts as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalReleasePolicy {
        GlobalReleasePolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            key_id: self.key_id.clone(),
            minimum: self.minimum,
            rate: self.rate,
            host: self.host.clone(),
            attempts: self.attempts,
            region: self.region.clone(),
            channel: self.channel.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a global release policy in status open.
pub fn new_global_release_policy(code: &str, label: &str) -> GlobalReleasePolicy {
    GlobalReleasePolicy {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        key_id: "".to_string(),
        minimum: 0.0,
        rate: 0.0,
        host: "".to_string(),
        attempts: 0,
        region: "".to_string(),
        channel: "".to_string(),
        issuer: "".to_string(),
        status: GlobalReleasePolicyStatus::Open,
    }
}
