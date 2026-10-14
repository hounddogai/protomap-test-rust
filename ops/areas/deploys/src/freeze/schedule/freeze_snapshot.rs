use super::*;

use acme_kit::text;

/// Where a freeze snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeSnapshotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FreezeSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FreezeSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One freeze snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSnapshot {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub description: String,
    pub budget: f64,
    pub ceiling: f64,
    pub burst: i64,
    pub status: FreezeSnapshotStatus,
}

impl FreezeSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeSnapshotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeSnapshotStatus) -> bool {
        match self.status {
            FreezeSnapshotStatus::Proposed => {
                return next == FreezeSnapshotStatus::Approved;
            }
            FreezeSnapshotStatus::Approved => {
                return next == FreezeSnapshotStatus::Rejected || next == FreezeSnapshotStatus::Withdrawn;
            }
            FreezeSnapshotStatus::Rejected => {
                match next {
                    FreezeSnapshotStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FreezeSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "freeze snapshot", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.budget * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeSnapshot {
        FreezeSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            description: self.description.clone(),
            budget: self.budget,
            ceiling: self.ceiling,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a freeze snapshot in status proposed.
pub fn new_freeze_snapshot(code: &str, title: &str) -> FreezeSnapshot {
    FreezeSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        description: "".to_string(),
        budget: 0.0,
        ceiling: 0.0,
        burst: 0,
        status: FreezeSnapshotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_freeze_snapshot("code-951", "title-640");
        assert_eq!(item.status, FreezeSnapshotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_freeze_snapshot("code-951", "title-640");
        if item.move_to(FreezeSnapshotStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeSnapshotStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_freeze_snapshot("code-951", "title-640");
        item.status = FreezeSnapshotStatus::Withdrawn;
        assert!(
            item.move_to(FreezeSnapshotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
