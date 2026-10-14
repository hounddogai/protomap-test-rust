use super::*;

use acme_kit::text;

/// Where a usage snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl UsageSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for UsageSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage snapshot as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageSnapshot {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub duration_minutes: i64,
    pub quota_units: i64,
    pub source: String,
    pub status: UsageSnapshotStatus,
}

impl UsageSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageSnapshotStatus) -> bool {
        match self.status {
            UsageSnapshotStatus::Created => {
                match next {
                    UsageSnapshotStatus::Validated | UsageSnapshotStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageSnapshotStatus::Validated => {
                return next == UsageSnapshotStatus::Published || next == UsageSnapshotStatus::Retired;
            }
            UsageSnapshotStatus::Published => {
                return next == UsageSnapshotStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "usage snapshot", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.quota_units as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageSnapshot {
        UsageSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            duration_minutes: self.duration_minutes,
            quota_units: self.quota_units,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a usage snapshot in status created.
pub fn new_usage_snapshot(code: &str, title: &str) -> UsageSnapshot {
    UsageSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        duration_minutes: 0,
        quota_units: 0,
        source: "".to_string(),
        status: UsageSnapshotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_usage_snapshot("code-372", "title-622");
        assert_eq!(item.status, UsageSnapshotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_usage_snapshot("code-372", "title-622");
        if item.move_to(UsageSnapshotStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageSnapshotStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_usage_snapshot("code-372", "title-622");
        item.status = UsageSnapshotStatus::Retired;
        assert!(
            item.move_to(UsageSnapshotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
