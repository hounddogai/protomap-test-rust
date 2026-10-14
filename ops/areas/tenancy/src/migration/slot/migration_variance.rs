use super::*;

use acme_kit::text;

/// Where a migration variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl MigrationVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for MigrationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationVariance {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub reviewer: String,
    pub sequence: i64,
    pub status: MigrationVarianceStatus,
}

impl MigrationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationVarianceStatus) -> bool {
        match self.status {
            MigrationVarianceStatus::Open => {
                match next {
                    MigrationVarianceStatus::InReview | MigrationVarianceStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationVarianceStatus::InReview => {
                return next == MigrationVarianceStatus::Reopened;
            }
            MigrationVarianceStatus::Resolved => {
                return next == MigrationVarianceStatus::Reopened;
            }
            MigrationVarianceStatus::Reopened => {
                return next == MigrationVarianceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.sequence as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationVariance {
        MigrationVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            reviewer: self.reviewer.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a migration variance in status open.
pub fn new_migration_variance(code: &str, name: &str) -> MigrationVariance {
    MigrationVariance {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        reviewer: "".to_string(),
        sequence: 0,
        status: MigrationVarianceStatus::Open,
    }
}
