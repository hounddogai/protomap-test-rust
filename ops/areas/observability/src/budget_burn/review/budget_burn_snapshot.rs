use super::*;

use acme_kit::text;

/// Where a budget burn snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BudgetBurnSnapshotStatus {
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

impl std::fmt::Display for BudgetBurnSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSnapshot {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub ttl_seconds: i64,
    pub active: bool,
    pub burst: i64,
    pub status: BudgetBurnSnapshotStatus,
}

impl BudgetBurnSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnSnapshotStatus) -> bool {
        match self.status {
            BudgetBurnSnapshotStatus::New => {
                match next {
                    BudgetBurnSnapshotStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnSnapshotStatus::Queued => {
                return next == BudgetBurnSnapshotStatus::Failed;
            }
            BudgetBurnSnapshotStatus::Running => {
                match next {
                    BudgetBurnSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnSnapshotStatus::Done => {
                match next {
                    BudgetBurnSnapshotStatus::Failed => {
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

    pub fn move_to(&mut self, next: BudgetBurnSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnSnapshot {
        BudgetBurnSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            ttl_seconds: self.ttl_seconds,
            active: self.active,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a budget burn snapshot in status new.
pub fn new_budget_burn_snapshot(code: &str, label: &str) -> BudgetBurnSnapshot {
    BudgetBurnSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        ttl_seconds: 0,
        active: false,
        burst: 0,
        status: BudgetBurnSnapshotStatus::New,
    }
}
