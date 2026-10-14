use super::*;

use acme_kit::text;

/// Where a scheduled egress policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledEgressPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledEgressPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ScheduledEgressPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled egress policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressPolicy {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub cluster: String,
    pub checksum: String,
    pub amount: f64,
    pub status: ScheduledEgressPolicyStatus,
}

impl ScheduledEgressPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledEgressPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledEgressPolicyStatus) -> bool {
        match self.status {
            ScheduledEgressPolicyStatus::New => {
                return next == ScheduledEgressPolicyStatus::Queued || next == ScheduledEgressPolicyStatus::Running;
            }
            ScheduledEgressPolicyStatus::Queued => {
                return next == ScheduledEgressPolicyStatus::Done;
            }
            ScheduledEgressPolicyStatus::Running => {
                return next == ScheduledEgressPolicyStatus::Done || next == ScheduledEgressPolicyStatus::Failed;
            }
            ScheduledEgressPolicyStatus::Done => {
                return next == ScheduledEgressPolicyStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledEgressPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled egress policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> ScheduledEgressPolicy {
        ScheduledEgressPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            cluster: self.cluster.clone(),
            checksum: self.checksum.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a scheduled egress policy in status new.
pub fn new_scheduled_egress_policy(code: &str, name: &str) -> ScheduledEgressPolicy {
    ScheduledEgressPolicy {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        cluster: "".to_string(),
        checksum: "".to_string(),
        amount: 0.0,
        status: ScheduledEgressPolicyStatus::New,
    }
}
