use super::*;

use acme_kit::text;

/// Where a rolling slo target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSloTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingSloTargetStatus {
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

impl std::fmt::Display for RollingSloTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling slo target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSloTarget {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub floor_value: f64,
    pub version: i64,
    pub notes: String,
    pub name: String,
    pub cluster: String,
    pub retries: i64,
    pub ceiling: f64,
    pub limit: i64,
    pub status: RollingSloTargetStatus,
}

impl RollingSloTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSloTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSloTargetStatus) -> bool {
        match self.status {
            RollingSloTargetStatus::New => {
                return next == RollingSloTargetStatus::Running || next == RollingSloTargetStatus::Failed;
            }
            RollingSloTargetStatus::Queued => {
                match next {
                    RollingSloTargetStatus::Done | RollingSloTargetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSloTargetStatus::Running => {
                return next == RollingSloTargetStatus::Failed;
            }
            RollingSloTargetStatus::Done => {
                match next {
                    RollingSloTargetStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingSloTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling slo target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling slo target", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.burst as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingSloTarget {
        RollingSloTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            floor_value: self.floor_value,
            version: self.version,
            notes: self.notes.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            retries: self.retries,
            ceiling: self.ceiling,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rolling slo target in status new.
pub fn new_rolling_slo_target(code: &str, title: &str) -> RollingSloTarget {
    RollingSloTarget {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        floor_value: 0.0,
        version: 0,
        notes: "".to_string(),
        name: "".to_string(),
        cluster: "".to_string(),
        retries: 0,
        ceiling: 0.0,
        limit: 0,
        status: RollingSloTargetStatus::New,
    }
}
