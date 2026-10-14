use super::*;

use acme_kit::text;

/// Where a tiered rollout proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRolloutProposalStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredRolloutProposalStatus {
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

impl std::fmt::Display for TieredRolloutProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered rollout proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutProposal {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub cipher: String,
    pub starts_at: i64,
    pub phone: String,
    pub key_id: String,
    pub reviewer: String,
    pub status: TieredRolloutProposalStatus,
}

impl TieredRolloutProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRolloutProposalStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRolloutProposalStatus) -> bool {
        match self.status {
            TieredRolloutProposalStatus::Open => {
                match next {
                    TieredRolloutProposalStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRolloutProposalStatus::InReview => {
                return next == TieredRolloutProposalStatus::Reopened || next == TieredRolloutProposalStatus::Archived;
            }
            TieredRolloutProposalStatus::Resolved => {
                return next == TieredRolloutProposalStatus::Archived;
            }
            TieredRolloutProposalStatus::Reopened => {
                return next == TieredRolloutProposalStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredRolloutProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered rollout proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered rollout proposal", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.starts_at as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRolloutProposal {
        TieredRolloutProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            cipher: self.cipher.clone(),
            starts_at: self.starts_at,
            phone: self.phone.clone(),
            key_id: self.key_id.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered rollout proposal in status open.
pub fn new_tiered_rollout_proposal(code: &str, label: &str) -> TieredRolloutProposal {
    TieredRolloutProposal {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        cipher: "".to_string(),
        starts_at: 0,
        phone: "".to_string(),
        key_id: "".to_string(),
        reviewer: "".to_string(),
        status: TieredRolloutProposalStatus::Open,
    }
}
