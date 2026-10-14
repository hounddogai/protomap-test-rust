use super::*;

use acme_kit::text;

/// Where a delegation rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DelegationRollupStatus {
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

impl std::fmt::Display for DelegationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation rollup as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRollup {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub summary: String,
    pub threshold: f64,
    pub starts_at: i64,
    pub budget: f64,
    pub status: DelegationRollupStatus,
}

impl DelegationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationRollupStatus) -> bool {
        match self.status {
            DelegationRollupStatus::New => {
                match next {
                    DelegationRollupStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationRollupStatus::Queued => {
                match next {
                    DelegationRollupStatus::Running | DelegationRollupStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationRollupStatus::Running => {
                match next {
                    DelegationRollupStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationRollupStatus::Done => {
                return next == DelegationRollupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation rollup {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.budget * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationRollup {
        DelegationRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            summary: self.summary.clone(),
            threshold: self.threshold,
            starts_at: self.starts_at,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a delegation rollup in status new.
pub fn new_delegation_rollup(code: &str, name: &str) -> DelegationRollup {
    DelegationRollup {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        summary: "".to_string(),
        threshold: 0.0,
        starts_at: 0,
        budget: 0.0,
        status: DelegationRollupStatus::New,
    }
}
