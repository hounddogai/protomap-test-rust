use super::*;

use acme_kit::text;

/// Where a fallback migration metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackMigrationMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackMigrationMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FallbackMigrationMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback migration metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationMetric {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub title: String,
    pub cluster: String,
    pub country: String,
    pub expires_at: i64,
    pub maximum: f64,
    pub cipher: String,
    pub category: String,
    pub source: String,
    pub status: FallbackMigrationMetricStatus,
}

impl FallbackMigrationMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackMigrationMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackMigrationMetricStatus) -> bool {
        match self.status {
            FallbackMigrationMetricStatus::Planned => {
                return next == FallbackMigrationMetricStatus::Scheduled
                    || next == FallbackMigrationMetricStatus::InProgress;
            }
            FallbackMigrationMetricStatus::Scheduled => {
                match next {
                    FallbackMigrationMetricStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackMigrationMetricStatus::InProgress => {
                match next {
                    FallbackMigrationMetricStatus::Completed | FallbackMigrationMetricStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackMigrationMetricStatus::Completed => {
                return next == FallbackMigrationMetricStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackMigrationMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback migration metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.expires_at as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackMigrationMetric {
        FallbackMigrationMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            country: self.country.clone(),
            expires_at: self.expires_at,
            maximum: self.maximum,
            cipher: self.cipher.clone(),
            category: self.category.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a fallback migration metric in status planned.
pub fn new_fallback_migration_metric(code: &str, name: &str) -> FallbackMigrationMetric {
    FallbackMigrationMetric {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        title: "".to_string(),
        cluster: "".to_string(),
        country: "".to_string(),
        expires_at: 0,
        maximum: 0.0,
        cipher: "".to_string(),
        category: "".to_string(),
        source: "".to_string(),
        status: FallbackMigrationMetricStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_migration_metric("code-515", "name-529");
        assert_eq!(item.status, FallbackMigrationMetricStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_fallback_migration_metric("code-515", "name-529");
        if item.move_to(FallbackMigrationMetricStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackMigrationMetricStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_migration_metric("code-515", "name-529");
        item.status = FallbackMigrationMetricStatus::Cancelled;
        assert!(
            item.move_to(FallbackMigrationMetricStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
