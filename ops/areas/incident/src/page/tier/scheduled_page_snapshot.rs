use super::*;

use acme_kit::text;

/// Where a scheduled page snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledPageSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledPageSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ScheduledPageSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled page snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageSnapshot {
    pub code: String,
    pub name: String,
    pub source: String,
    pub approved: bool,
    pub version: i64,
    pub cost: f64,
    pub capacity: i64,
    pub score: f64,
    pub weight: f64,
    pub status: ScheduledPageSnapshotStatus,
}

impl ScheduledPageSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledPageSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledPageSnapshotStatus) -> bool {
        match self.status {
            ScheduledPageSnapshotStatus::Idle => {
                match next {
                    ScheduledPageSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPageSnapshotStatus::Warming => {
                return next == ScheduledPageSnapshotStatus::Ready || next == ScheduledPageSnapshotStatus::Stopped;
            }
            ScheduledPageSnapshotStatus::Ready => {
                match next {
                    ScheduledPageSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledPageSnapshotStatus::Draining => {
                match next {
                    ScheduledPageSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledPageSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled page snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.cost - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledPageSnapshot {
        ScheduledPageSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            approved: self.approved,
            version: self.version,
            cost: self.cost,
            capacity: self.capacity,
            score: self.score,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a scheduled page snapshot in status idle.
pub fn new_scheduled_page_snapshot(code: &str, name: &str) -> ScheduledPageSnapshot {
    ScheduledPageSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        approved: false,
        version: 0,
        cost: 0.0,
        capacity: 0,
        score: 0.0,
        weight: 0.0,
        status: ScheduledPageSnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_page_snapshot("code-277", "name-496");
        assert_eq!(item.status, ScheduledPageSnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_scheduled_page_snapshot("code-277", "name-496");
        if item.move_to(ScheduledPageSnapshotStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledPageSnapshotStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_page_snapshot("code-277", "name-496");
        item.status = ScheduledPageSnapshotStatus::Stopped;
        assert!(
            item.move_to(ScheduledPageSnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
