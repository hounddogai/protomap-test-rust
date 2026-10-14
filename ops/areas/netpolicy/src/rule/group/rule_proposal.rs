use super::*;

use acme_kit::text;

/// Where a rule proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RuleProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RuleProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule proposal as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleProposal {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub starts_at: i64,
    pub segment: String,
    pub cluster: String,
    pub maximum: f64,
    pub external_ref: String,
    pub status: RuleProposalStatus,
}

impl RuleProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleProposalStatus) -> bool {
        match self.status {
            RuleProposalStatus::Planned => {
                match next {
                    RuleProposalStatus::Scheduled | RuleProposalStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleProposalStatus::Scheduled => {
                return next == RuleProposalStatus::Cancelled;
            }
            RuleProposalStatus::InProgress => {
                return next == RuleProposalStatus::Completed || next == RuleProposalStatus::Cancelled;
            }
            RuleProposalStatus::Completed => {
                return next == RuleProposalStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule proposal {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.maximum - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleProposal {
        RuleProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            starts_at: self.starts_at,
            segment: self.segment.clone(),
            cluster: self.cluster.clone(),
            maximum: self.maximum,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rule proposal in status planned.
pub fn new_rule_proposal(code: &str, title: &str) -> RuleProposal {
    RuleProposal {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        starts_at: 0,
        segment: "".to_string(),
        cluster: "".to_string(),
        maximum: 0.0,
        external_ref: "".to_string(),
        status: RuleProposalStatus::Planned,
    }
}
