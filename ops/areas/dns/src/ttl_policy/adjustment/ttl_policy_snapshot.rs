use super::*;

use acme_kit::text;

/// Where a ttl policy snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicySnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TtlPolicySnapshotStatus {
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

impl std::fmt::Display for TtlPolicySnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a ttl policy snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicySnapshot {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub issuer: String,
    pub budget: f64,
    pub notes: String,
    pub attempts: i64,
    pub owner_id: String,
    pub status: TtlPolicySnapshotStatus,
}

impl TtlPolicySnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicySnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicySnapshotStatus) -> bool {
        match self.status {
            TtlPolicySnapshotStatus::Planned => {
                match next {
                    TtlPolicySnapshotStatus::Completed | TtlPolicySnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicySnapshotStatus::Scheduled => {
                match next {
                    TtlPolicySnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicySnapshotStatus::InProgress => {
                return next == TtlPolicySnapshotStatus::Cancelled;
            }
            TtlPolicySnapshotStatus::Completed => {
                match next {
                    TtlPolicySnapshotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TtlPolicySnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.created_at as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicySnapshot {
        TtlPolicySnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            budget: self.budget,
            notes: self.notes.clone(),
            attempts: self.attempts,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a ttl policy snapshot in status planned.
pub fn new_ttl_policy_snapshot(code: &str, name: &str) -> TtlPolicySnapshot {
    TtlPolicySnapshot {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        issuer: "".to_string(),
        budget: 0.0,
        notes: "".to_string(),
        attempts: 0,
        owner_id: "".to_string(),
        status: TtlPolicySnapshotStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_ttl_policy_snapshot("code-291", "name-662");
        assert_eq!(item.status, TtlPolicySnapshotStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_ttl_policy_snapshot("code-291", "name-662");
        if item.move_to(TtlPolicySnapshotStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicySnapshotStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_ttl_policy_snapshot("code-291", "name-662");
        item.status = TtlPolicySnapshotStatus::Cancelled;
        assert!(
            item.move_to(TtlPolicySnapshotStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
