use super::*;

use acme_kit::text;

/// Where a legacy rollout proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRolloutProposalStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyRolloutProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyRolloutProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy rollout proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutProposal {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub created_at: i64,
    pub quota_units: i64,
    pub checksum: String,
    pub duration_minutes: i64,
    pub title: String,
    pub pinned: bool,
    pub status: LegacyRolloutProposalStatus,
}

impl LegacyRolloutProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRolloutProposalStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRolloutProposalStatus) -> bool {
        match self.status {
            LegacyRolloutProposalStatus::Proposed => {
                return next == LegacyRolloutProposalStatus::Withdrawn;
            }
            LegacyRolloutProposalStatus::Approved => {
                return next == LegacyRolloutProposalStatus::Withdrawn;
            }
            LegacyRolloutProposalStatus::Rejected => {
                match next {
                    LegacyRolloutProposalStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LegacyRolloutProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rollout proposal {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.quota_units as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRolloutProposal {
        LegacyRolloutProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            created_at: self.created_at,
            quota_units: self.quota_units,
            checksum: self.checksum.clone(),
            duration_minutes: self.duration_minutes,
            title: self.title.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy rollout proposal in status proposed.
pub fn new_legacy_rollout_proposal(code: &str, name: &str) -> LegacyRolloutProposal {
    LegacyRolloutProposal {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        created_at: 0,
        quota_units: 0,
        checksum: "".to_string(),
        duration_minutes: 0,
        title: "".to_string(),
        pinned: false,
        status: LegacyRolloutProposalStatus::Proposed,
    }
}
