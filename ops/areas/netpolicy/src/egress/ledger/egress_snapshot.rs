use super::*;

use acme_kit::text;

/// Where a egress snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressSnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EgressSnapshotStatus {
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

impl std::fmt::Display for EgressSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress snapshot as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressSnapshot {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub title: String,
    pub duration_minutes: i64,
    pub threshold: f64,
    pub owner_id: String,
    pub cost: f64,
    pub maximum: f64,
    pub region: String,
    pub status: EgressSnapshotStatus,
}

impl EgressSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressSnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressSnapshotStatus) -> bool {
        match self.status {
            EgressSnapshotStatus::Planned => {
                match next {
                    EgressSnapshotStatus::InProgress | EgressSnapshotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressSnapshotStatus::Scheduled => {
                match next {
                    EgressSnapshotStatus::Completed | EgressSnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressSnapshotStatus::InProgress => {
                return next == EgressSnapshotStatus::Completed;
            }
            EgressSnapshotStatus::Completed => {
                return next == EgressSnapshotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.duration_minutes as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressSnapshot {
        EgressSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            threshold: self.threshold,
            owner_id: self.owner_id.clone(),
            cost: self.cost,
            maximum: self.maximum,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a egress snapshot in status planned.
pub fn new_egress_snapshot(code: &str, name: &str) -> EgressSnapshot {
    EgressSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        title: "".to_string(),
        duration_minutes: 0,
        threshold: 0.0,
        owner_id: "".to_string(),
        cost: 0.0,
        maximum: 0.0,
        region: "".to_string(),
        status: EgressSnapshotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_egress_snapshot("code-302", "name-762");
        assert_eq!(item.status, EgressSnapshotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_egress_snapshot("code-302", "name-762");
        if item.move_to(EgressSnapshotStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressSnapshotStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_egress_snapshot("code-302", "name-762");
        item.status = EgressSnapshotStatus::Cancelled;
        assert!(
            item.move_to(EgressSnapshotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
