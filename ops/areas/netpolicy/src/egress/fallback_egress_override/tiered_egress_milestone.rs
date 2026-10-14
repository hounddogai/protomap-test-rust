use super::*;

use acme_kit::text;

/// Where a tiered egress milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredEgressMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredEgressMilestoneStatus {
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

impl std::fmt::Display for TieredEgressMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered egress milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressMilestone {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub host: String,
    pub grace_days: i64,
    pub revision: i64,
    pub minimum: f64,
    pub checksum: String,
    pub score: f64,
    pub version: i64,
    pub rate: f64,
    pub status: TieredEgressMilestoneStatus,
}

impl TieredEgressMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredEgressMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredEgressMilestoneStatus) -> bool {
        match self.status {
            TieredEgressMilestoneStatus::New => {
                return next == TieredEgressMilestoneStatus::Done || next == TieredEgressMilestoneStatus::Failed;
            }
            TieredEgressMilestoneStatus::Queued => {
                match next {
                    TieredEgressMilestoneStatus::Running | TieredEgressMilestoneStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredEgressMilestoneStatus::Running => {
                return next == TieredEgressMilestoneStatus::Done || next == TieredEgressMilestoneStatus::Failed;
            }
            TieredEgressMilestoneStatus::Done => {
                match next {
                    TieredEgressMilestoneStatus::Failed => {
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

    pub fn move_to(&mut self, next: TieredEgressMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered egress milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        (self.score * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredEgressMilestone {
        TieredEgressMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            host: self.host.clone(),
            grace_days: self.grace_days,
            revision: self.revision,
            minimum: self.minimum,
            checksum: self.checksum.clone(),
            score: self.score,
            version: self.version,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered egress milestone in status new.
pub fn new_tiered_egress_milestone(code: &str, name: &str) -> TieredEgressMilestone {
    TieredEgressMilestone {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        host: "".to_string(),
        grace_days: 0,
        revision: 0,
        minimum: 0.0,
        checksum: "".to_string(),
        score: 0.0,
        version: 0,
        rate: 0.0,
        status: TieredEgressMilestoneStatus::New,
    }
}
