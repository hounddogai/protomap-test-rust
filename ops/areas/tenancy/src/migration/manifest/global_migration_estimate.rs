use super::*;

use acme_kit::text;

/// Where a global migration estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalMigrationEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalMigrationEstimateStatus {
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

impl std::fmt::Display for GlobalMigrationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global migration estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMigrationEstimate {
    pub code: String,
    pub title: String,
    pub label: String,
    pub ceiling: f64,
    pub port: i64,
    pub phone: String,
    pub status: GlobalMigrationEstimateStatus,
}

impl GlobalMigrationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalMigrationEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalMigrationEstimateStatus) -> bool {
        match self.status {
            GlobalMigrationEstimateStatus::New => {
                match next {
                    GlobalMigrationEstimateStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalMigrationEstimateStatus::Queued => {
                match next {
                    GlobalMigrationEstimateStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalMigrationEstimateStatus::Running => {
                match next {
                    GlobalMigrationEstimateStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalMigrationEstimateStatus::Done => {
                return next == GlobalMigrationEstimateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalMigrationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global migration estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global migration estimate", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ceiling + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalMigrationEstimate {
        GlobalMigrationEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            port: self.port,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global migration estimate in status new.
pub fn new_global_migration_estimate(code: &str, title: &str) -> GlobalMigrationEstimate {
    GlobalMigrationEstimate {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        ceiling: 0.0,
        port: 0,
        phone: "".to_string(),
        status: GlobalMigrationEstimateStatus::New,
    }
}
