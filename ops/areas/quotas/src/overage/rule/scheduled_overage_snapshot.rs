use super::*;

use acme_kit::text;

/// Where a scheduled overage snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledOverageSnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledOverageSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ScheduledOverageSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled overage snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageSnapshot {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub limit: i64,
    pub rate: f64,
    pub weight: f64,
    pub active: bool,
    pub checksum: String,
    pub discount: f64,
    pub status: ScheduledOverageSnapshotStatus,
}

impl ScheduledOverageSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledOverageSnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledOverageSnapshotStatus) -> bool {
        match self.status {
            ScheduledOverageSnapshotStatus::Draft => {
                match next {
                    ScheduledOverageSnapshotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledOverageSnapshotStatus::Pending => {
                match next {
                    ScheduledOverageSnapshotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledOverageSnapshotStatus::Active => {
                return next == ScheduledOverageSnapshotStatus::Suspended
                    || next == ScheduledOverageSnapshotStatus::Closed;
            }
            ScheduledOverageSnapshotStatus::Suspended => {
                return next == ScheduledOverageSnapshotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledOverageSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled overage snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled overage snapshot", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.discount * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledOverageSnapshot {
        ScheduledOverageSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            limit: self.limit,
            rate: self.rate,
            weight: self.weight,
            active: self.active,
            checksum: self.checksum.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled overage snapshot in status draft.
pub fn new_scheduled_overage_snapshot(code: &str, name: &str) -> ScheduledOverageSnapshot {
    ScheduledOverageSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        limit: 0,
        rate: 0.0,
        weight: 0.0,
        active: false,
        checksum: "".to_string(),
        discount: 0.0,
        status: ScheduledOverageSnapshotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_overage_snapshot("code-696", "name-339");
        assert_eq!(item.status, ScheduledOverageSnapshotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_scheduled_overage_snapshot("code-696", "name-339");
        if item.move_to(ScheduledOverageSnapshotStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledOverageSnapshotStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_overage_snapshot("code-696", "name-339");
        item.status = ScheduledOverageSnapshotStatus::Closed;
        assert!(
            item.move_to(ScheduledOverageSnapshotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
