use super::*;

use acme_kit::text;

/// Where a slo snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SloSnapshotStatus {
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

impl std::fmt::Display for SloSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One slo snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloSnapshot {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub duration_minutes: i64,
    pub fingerprint: String,
    pub audience: String,
    pub limit: i64,
    pub approved: bool,
    pub priority: i64,
    pub timezone: String,
    pub ceiling: f64,
    pub status: SloSnapshotStatus,
}

impl SloSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloSnapshotStatus) -> bool {
        match self.status {
            SloSnapshotStatus::Planned => {
                match next {
                    SloSnapshotStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloSnapshotStatus::Scheduled => {
                match next {
                    SloSnapshotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloSnapshotStatus::InProgress => {
                return next == SloSnapshotStatus::Completed || next == SloSnapshotStatus::Cancelled;
            }
            SloSnapshotStatus::Completed => {
                return next == SloSnapshotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ceiling - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloSnapshot {
        SloSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            duration_minutes: self.duration_minutes,
            fingerprint: self.fingerprint.clone(),
            audience: self.audience.clone(),
            limit: self.limit,
            approved: self.approved,
            priority: self.priority,
            timezone: self.timezone.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a slo snapshot in status planned.
pub fn new_slo_snapshot(code: &str, label: &str) -> SloSnapshot {
    SloSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        duration_minutes: 0,
        fingerprint: "".to_string(),
        audience: "".to_string(),
        limit: 0,
        approved: false,
        priority: 0,
        timezone: "".to_string(),
        ceiling: 0.0,
        status: SloSnapshotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_slo_snapshot("code-205", "label-969");
        assert_eq!(item.status, SloSnapshotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_slo_snapshot("code-205", "label-969");
        if item.move_to(SloSnapshotStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloSnapshotStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_slo_snapshot("code-205", "label-969");
        item.status = SloSnapshotStatus::Cancelled;
        assert!(
            item.move_to(SloSnapshotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
