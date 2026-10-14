use super::*;

use acme_kit::text;

/// Where a pending allowlist claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingAllowlistClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingAllowlistClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingAllowlistClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending allowlist claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingAllowlistClaim {
    pub code: String,
    pub label: String,
    pub source: String,
    pub retries: i64,
    pub floor_value: f64,
    pub notes: String,
    pub country: String,
    pub ttl_seconds: i64,
    pub channel: String,
    pub status: PendingAllowlistClaimStatus,
}

impl PendingAllowlistClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingAllowlistClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingAllowlistClaimStatus) -> bool {
        match self.status {
            PendingAllowlistClaimStatus::Proposed => {
                return next == PendingAllowlistClaimStatus::Approved;
            }
            PendingAllowlistClaimStatus::Approved => {
                match next {
                    PendingAllowlistClaimStatus::Rejected | PendingAllowlistClaimStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingAllowlistClaimStatus::Rejected => {
                match next {
                    PendingAllowlistClaimStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingAllowlistClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending allowlist claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.retries as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingAllowlistClaim {
        PendingAllowlistClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            retries: self.retries,
            floor_value: self.floor_value,
            notes: self.notes.clone(),
            country: self.country.clone(),
            ttl_seconds: self.ttl_seconds,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending allowlist claim in status proposed.
pub fn new_pending_allowlist_claim(code: &str, label: &str) -> PendingAllowlistClaim {
    PendingAllowlistClaim {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        retries: 0,
        floor_value: 0.0,
        notes: "".to_string(),
        country: "".to_string(),
        ttl_seconds: 0,
        channel: "".to_string(),
        status: PendingAllowlistClaimStatus::Proposed,
    }
}
