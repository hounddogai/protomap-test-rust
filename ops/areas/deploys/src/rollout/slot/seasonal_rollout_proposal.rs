use super::*;

use acme_kit::text;

/// Where a seasonal rollout proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRolloutProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalRolloutProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalRolloutProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal rollout proposal as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutProposal {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub host: String,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub ratio: f64,
    pub scope: String,
    pub tenant_id: String,
    pub rotation_days: i64,
    pub source: String,
    pub status: SeasonalRolloutProposalStatus,
}

impl SeasonalRolloutProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRolloutProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRolloutProposalStatus) -> bool {
        match self.status {
            SeasonalRolloutProposalStatus::Created => {
                return next == SeasonalRolloutProposalStatus::Published;
            }
            SeasonalRolloutProposalStatus::Validated => {
                match next {
                    SeasonalRolloutProposalStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRolloutProposalStatus::Published => {
                return next == SeasonalRolloutProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalRolloutProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal rollout proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        (self.ratio * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRolloutProposal {
        SeasonalRolloutProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            host: self.host.clone(),
            pinned: self.pinned,
            tags: self.tags.clone(),
            ratio: self.ratio,
            scope: self.scope.clone(),
            tenant_id: self.tenant_id.clone(),
            rotation_days: self.rotation_days,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a seasonal rollout proposal in status created.
pub fn new_seasonal_rollout_proposal(code: &str, name: &str) -> SeasonalRolloutProposal {
    SeasonalRolloutProposal {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        host: "".to_string(),
        pinned: false,
        tags: Vec::new(),
        ratio: 0.0,
        scope: "".to_string(),
        tenant_id: "".to_string(),
        rotation_days: 0,
        source: "".to_string(),
        status: SeasonalRolloutProposalStatus::Created,
    }
}
