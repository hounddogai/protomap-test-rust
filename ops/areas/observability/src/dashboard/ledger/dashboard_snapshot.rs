use super::*;

use acme_kit::text;

/// Where a dashboard snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DashboardSnapshotStatus {
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

impl std::fmt::Display for DashboardSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard snapshot as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSnapshot {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub issuer: String,
    pub limit: i64,
    pub grace_days: i64,
    pub status: DashboardSnapshotStatus,
}

impl DashboardSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardSnapshotStatus) -> bool {
        match self.status {
            DashboardSnapshotStatus::New => {
                return next == DashboardSnapshotStatus::Queued || next == DashboardSnapshotStatus::Failed;
            }
            DashboardSnapshotStatus::Queued => {
                match next {
                    DashboardSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardSnapshotStatus::Running => {
                match next {
                    DashboardSnapshotStatus::Done | DashboardSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardSnapshotStatus::Done => {
                return next == DashboardSnapshotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.limit as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardSnapshot {
        DashboardSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            issuer: self.issuer.clone(),
            limit: self.limit,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dashboard snapshot in status new.
pub fn new_dashboard_snapshot(code: &str, label: &str) -> DashboardSnapshot {
    DashboardSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        issuer: "".to_string(),
        limit: 0,
        grace_days: 0,
        status: DashboardSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_dashboard_snapshot("code-231", "label-861");
        assert_eq!(item.status, DashboardSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_dashboard_snapshot("code-231", "label-861");
        if item.move_to(DashboardSnapshotStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardSnapshotStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_dashboard_snapshot("code-231", "label-861");
        item.status = DashboardSnapshotStatus::Failed;
        assert!(
            item.move_to(DashboardSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
