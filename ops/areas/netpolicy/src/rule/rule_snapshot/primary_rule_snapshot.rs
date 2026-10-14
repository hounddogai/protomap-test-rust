use super::*;

use acme_kit::text;

/// Where a primary rule snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRuleSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryRuleSnapshotStatus {
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

impl std::fmt::Display for PrimaryRuleSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary rule snapshot as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleSnapshot {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub duration_minutes: i64,
    pub limit: i64,
    pub status: PrimaryRuleSnapshotStatus,
}

impl PrimaryRuleSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRuleSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRuleSnapshotStatus) -> bool {
        match self.status {
            PrimaryRuleSnapshotStatus::Planned => {
                return next == PrimaryRuleSnapshotStatus::Cancelled;
            }
            PrimaryRuleSnapshotStatus::Scheduled => {
                return next == PrimaryRuleSnapshotStatus::InProgress || next == PrimaryRuleSnapshotStatus::Cancelled;
            }
            PrimaryRuleSnapshotStatus::InProgress => {
                return next == PrimaryRuleSnapshotStatus::Completed || next == PrimaryRuleSnapshotStatus::Cancelled;
            }
            PrimaryRuleSnapshotStatus::Completed => {
                return next == PrimaryRuleSnapshotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryRuleSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rule snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary rule snapshot", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.limit as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRuleSnapshot {
        PrimaryRuleSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            duration_minutes: self.duration_minutes,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary rule snapshot in status planned.
pub fn new_primary_rule_snapshot(code: &str, title: &str) -> PrimaryRuleSnapshot {
    PrimaryRuleSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        duration_minutes: 0,
        limit: 0,
        status: PrimaryRuleSnapshotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_rule_snapshot("code-875", "title-349");
        assert_eq!(item.status, PrimaryRuleSnapshotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_primary_rule_snapshot("code-875", "title-349");
        if item.move_to(PrimaryRuleSnapshotStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRuleSnapshotStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_rule_snapshot("code-875", "title-349");
        item.status = PrimaryRuleSnapshotStatus::Cancelled;
        assert!(
            item.move_to(PrimaryRuleSnapshotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
