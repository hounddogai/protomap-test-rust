use super::*;

use acme_kit::text;

/// Where a retention snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionSnapshotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RetentionSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RetentionSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a retention snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSnapshot {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub reviewer: String,
    pub discount: f64,
    pub threshold: f64,
    pub status: RetentionSnapshotStatus,
}

impl RetentionSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionSnapshotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionSnapshotStatus) -> bool {
        match self.status {
            RetentionSnapshotStatus::Proposed => {
                match next {
                    RetentionSnapshotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionSnapshotStatus::Approved => {
                match next {
                    RetentionSnapshotStatus::Rejected | RetentionSnapshotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionSnapshotStatus::Rejected => {
                return next == RetentionSnapshotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.threshold + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionSnapshot {
        RetentionSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            reviewer: self.reviewer.clone(),
            discount: self.discount,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a retention snapshot in status proposed.
pub fn new_retention_snapshot(code: &str, name: &str) -> RetentionSnapshot {
    RetentionSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        reviewer: "".to_string(),
        discount: 0.0,
        threshold: 0.0,
        status: RetentionSnapshotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_retention_snapshot("code-767", "name-9");
        assert_eq!(item.status, RetentionSnapshotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_retention_snapshot("code-767", "name-9");
        if item.move_to(RetentionSnapshotStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionSnapshotStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_retention_snapshot("code-767", "name-9");
        item.status = RetentionSnapshotStatus::Withdrawn;
        assert!(
            item.move_to(RetentionSnapshotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
