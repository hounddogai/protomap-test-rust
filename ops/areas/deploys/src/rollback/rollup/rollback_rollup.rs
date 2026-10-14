use super::*;

use acme_kit::text;

/// Where a rollback rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollbackRollupStatus {
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

impl std::fmt::Display for RollbackRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback rollup as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRollup {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub version: i64,
    pub locale: String,
    pub external_ref: String,
    pub reviewer: String,
    pub currency: String,
    pub scope: String,
    pub limit: i64,
    pub discount: f64,
    pub status: RollbackRollupStatus,
}

impl RollbackRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackRollupStatus) -> bool {
        match self.status {
            RollbackRollupStatus::New => {
                return next == RollbackRollupStatus::Queued;
            }
            RollbackRollupStatus::Queued => {
                match next {
                    RollbackRollupStatus::Done | RollbackRollupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackRollupStatus::Running => {
                match next {
                    RollbackRollupStatus::Done | RollbackRollupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackRollupStatus::Done => {
                return next == RollbackRollupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.limit as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackRollup {
        RollbackRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            version: self.version,
            locale: self.locale.clone(),
            external_ref: self.external_ref.clone(),
            reviewer: self.reviewer.clone(),
            currency: self.currency.clone(),
            scope: self.scope.clone(),
            limit: self.limit,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rollback rollup in status new.
pub fn new_rollback_rollup(code: &str, title: &str) -> RollbackRollup {
    RollbackRollup {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        version: 0,
        locale: "".to_string(),
        external_ref: "".to_string(),
        reviewer: "".to_string(),
        currency: "".to_string(),
        scope: "".to_string(),
        limit: 0,
        discount: 0.0,
        status: RollbackRollupStatus::New,
    }
}
