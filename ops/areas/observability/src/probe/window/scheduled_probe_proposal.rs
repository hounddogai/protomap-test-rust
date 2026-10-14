use super::*;

use acme_kit::text;

/// Where a scheduled probe proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledProbeProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledProbeProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledProbeProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled probe proposal as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeProposal {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub ends_at: i64,
    pub ratio: f64,
    pub status: ScheduledProbeProposalStatus,
}

impl ScheduledProbeProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledProbeProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledProbeProposalStatus) -> bool {
        match self.status {
            ScheduledProbeProposalStatus::Created => {
                match next {
                    ScheduledProbeProposalStatus::Validated | ScheduledProbeProposalStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledProbeProposalStatus::Validated => {
                match next {
                    ScheduledProbeProposalStatus::Published | ScheduledProbeProposalStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledProbeProposalStatus::Published => {
                return next == ScheduledProbeProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledProbeProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled probe proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled probe proposal", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ratio + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledProbeProposal {
        ScheduledProbeProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            ends_at: self.ends_at,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a scheduled probe proposal in status created.
pub fn new_scheduled_probe_proposal(code: &str, label: &str) -> ScheduledProbeProposal {
    ScheduledProbeProposal {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        ends_at: 0,
        ratio: 0.0,
        status: ScheduledProbeProposalStatus::Created,
    }
}
