use super::*;

use acme_kit::text;

/// Where a adaptive rollout snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRolloutSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveRolloutSnapshotStatus {
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

impl std::fmt::Display for AdaptiveRolloutSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive rollout snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutSnapshot {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub duration_minutes: i64,
    pub cost: f64,
    pub discount: f64,
    pub status: AdaptiveRolloutSnapshotStatus,
}

impl AdaptiveRolloutSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRolloutSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRolloutSnapshotStatus) -> bool {
        match self.status {
            AdaptiveRolloutSnapshotStatus::Planned => {
                return next == AdaptiveRolloutSnapshotStatus::Scheduled
                    || next == AdaptiveRolloutSnapshotStatus::InProgress;
            }
            AdaptiveRolloutSnapshotStatus::Scheduled => {
                match next {
                    AdaptiveRolloutSnapshotStatus::InProgress | AdaptiveRolloutSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRolloutSnapshotStatus::InProgress => {
                match next {
                    AdaptiveRolloutSnapshotStatus::Completed | AdaptiveRolloutSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRolloutSnapshotStatus::Completed => {
                return next == AdaptiveRolloutSnapshotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRolloutSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rollout snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.discount - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRolloutSnapshot {
        AdaptiveRolloutSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            duration_minutes: self.duration_minutes,
            cost: self.cost,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive rollout snapshot in status planned.
pub fn new_adaptive_rollout_snapshot(code: &str, title: &str) -> AdaptiveRolloutSnapshot {
    AdaptiveRolloutSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        duration_minutes: 0,
        cost: 0.0,
        discount: 0.0,
        status: AdaptiveRolloutSnapshotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_adaptive_rollout_snapshot("code-736", "title-556");
        assert_eq!(item.status, AdaptiveRolloutSnapshotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_adaptive_rollout_snapshot("code-736", "title-556");
        if item.move_to(AdaptiveRolloutSnapshotStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRolloutSnapshotStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_adaptive_rollout_snapshot("code-736", "title-556");
        item.status = AdaptiveRolloutSnapshotStatus::Cancelled;
        assert!(
            item.move_to(AdaptiveRolloutSnapshotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
