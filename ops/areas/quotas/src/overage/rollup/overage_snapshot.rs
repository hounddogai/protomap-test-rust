use super::*;

use acme_kit::text;

/// Where a overage snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl OverageSnapshotStatus {
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

impl std::fmt::Display for OverageSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One overage snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageSnapshot {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub ends_at: i64,
    pub scope: String,
    pub threshold: f64,
    pub phone: String,
    pub amount: f64,
    pub label: String,
    pub summary: String,
    pub window_days: i64,
    pub status: OverageSnapshotStatus,
}

impl OverageSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageSnapshotStatus) -> bool {
        match self.status {
            OverageSnapshotStatus::Planned => {
                match next {
                    OverageSnapshotStatus::Scheduled | OverageSnapshotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageSnapshotStatus::Scheduled => {
                match next {
                    OverageSnapshotStatus::Completed | OverageSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageSnapshotStatus::InProgress => {
                match next {
                    OverageSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageSnapshotStatus::Completed => {
                return next == OverageSnapshotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage snapshot {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.threshold - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageSnapshot {
        OverageSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            ends_at: self.ends_at,
            scope: self.scope.clone(),
            threshold: self.threshold,
            phone: self.phone.clone(),
            amount: self.amount,
            label: self.label.clone(),
            summary: self.summary.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a overage snapshot in status planned.
pub fn new_overage_snapshot(code: &str, title: &str) -> OverageSnapshot {
    OverageSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        ends_at: 0,
        scope: "".to_string(),
        threshold: 0.0,
        phone: "".to_string(),
        amount: 0.0,
        label: "".to_string(),
        summary: "".to_string(),
        window_days: 0,
        status: OverageSnapshotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_overage_snapshot("code-824", "title-376");
        assert_eq!(item.status, OverageSnapshotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_overage_snapshot("code-824", "title-376");
        if item.move_to(OverageSnapshotStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageSnapshotStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_overage_snapshot("code-824", "title-376");
        item.status = OverageSnapshotStatus::Cancelled;
        assert!(
            item.move_to(OverageSnapshotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
