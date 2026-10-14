use super::*;

use acme_kit::text;

/// Where a default migration summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultMigrationSummaryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultMigrationSummaryStatus {
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

impl std::fmt::Display for DefaultMigrationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default migration summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationSummary {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub reviewer: String,
    pub audience: String,
    pub version: i64,
    pub updated_at: i64,
    pub status: DefaultMigrationSummaryStatus,
}

impl DefaultMigrationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultMigrationSummaryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultMigrationSummaryStatus) -> bool {
        match self.status {
            DefaultMigrationSummaryStatus::New => {
                match next {
                    DefaultMigrationSummaryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMigrationSummaryStatus::Queued => {
                return next == DefaultMigrationSummaryStatus::Done;
            }
            DefaultMigrationSummaryStatus::Running => {
                return next == DefaultMigrationSummaryStatus::Failed;
            }
            DefaultMigrationSummaryStatus::Done => {
                return next == DefaultMigrationSummaryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultMigrationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default migration summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.version as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultMigrationSummary {
        DefaultMigrationSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            reviewer: self.reviewer.clone(),
            audience: self.audience.clone(),
            version: self.version,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a default migration summary in status new.
pub fn new_default_migration_summary(code: &str, name: &str) -> DefaultMigrationSummary {
    DefaultMigrationSummary {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        reviewer: "".to_string(),
        audience: "".to_string(),
        version: 0,
        updated_at: 0,
        status: DefaultMigrationSummaryStatus::New,
    }
}
