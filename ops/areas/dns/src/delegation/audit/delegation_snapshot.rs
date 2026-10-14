use super::*;

use acme_kit::text;

/// Where a delegation snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationSnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DelegationSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DelegationSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationSnapshot {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub issuer: String,
    pub margin: f64,
    pub retries: i64,
    pub sequence: i64,
    pub maximum: f64,
    pub floor_value: f64,
    pub status: DelegationSnapshotStatus,
}

impl DelegationSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationSnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationSnapshotStatus) -> bool {
        match self.status {
            DelegationSnapshotStatus::Open => {
                match next {
                    DelegationSnapshotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationSnapshotStatus::InReview => {
                return next == DelegationSnapshotStatus::Resolved || next == DelegationSnapshotStatus::Archived;
            }
            DelegationSnapshotStatus::Resolved => {
                match next {
                    DelegationSnapshotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationSnapshotStatus::Reopened => {
                return next == DelegationSnapshotStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.retries as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationSnapshot {
        DelegationSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            issuer: self.issuer.clone(),
            margin: self.margin,
            retries: self.retries,
            sequence: self.sequence,
            maximum: self.maximum,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a delegation snapshot in status open.
pub fn new_delegation_snapshot(code: &str, name: &str) -> DelegationSnapshot {
    DelegationSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        issuer: "".to_string(),
        margin: 0.0,
        retries: 0,
        sequence: 0,
        maximum: 0.0,
        floor_value: 0.0,
        status: DelegationSnapshotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_delegation_snapshot("code-783", "name-345");
        assert_eq!(item.status, DelegationSnapshotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_delegation_snapshot("code-783", "name-345");
        if item.move_to(DelegationSnapshotStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationSnapshotStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_delegation_snapshot("code-783", "name-345");
        item.status = DelegationSnapshotStatus::Archived;
        assert!(
            item.move_to(DelegationSnapshotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
