use super::*;

use acme_kit::text;

/// Where a rollback cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackCycleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollbackCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollbackCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollback cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCycle {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub title: String,
    pub archived: bool,
    pub approved: bool,
    pub capacity: i64,
    pub status: RollbackCycleStatus,
}

impl RollbackCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackCycleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackCycleStatus) -> bool {
        match self.status {
            RollbackCycleStatus::Requested => {
                return next == RollbackCycleStatus::Granted || next == RollbackCycleStatus::Revoked;
            }
            RollbackCycleStatus::Granted => {
                match next {
                    RollbackCycleStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackCycleStatus::Revoked => {
                return next == RollbackCycleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> RollbackCycle {
        RollbackCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            title: self.title.clone(),
            archived: self.archived,
            approved: self.approved,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rollback cycle in status requested.
pub fn new_rollback_cycle(code: &str, name: &str) -> RollbackCycle {
    RollbackCycle {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        title: "".to_string(),
        archived: false,
        approved: false,
        capacity: 0,
        status: RollbackCycleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollback_cycle("code-900", "name-28");
        assert_eq!(item.status, RollbackCycleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rollback_cycle("code-900", "name-28");
        if item.move_to(RollbackCycleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackCycleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollback_cycle("code-900", "name-28");
        item.status = RollbackCycleStatus::Expired;
        assert!(
            item.move_to(RollbackCycleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
