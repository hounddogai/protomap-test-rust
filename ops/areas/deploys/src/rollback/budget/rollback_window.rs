use super::*;

use acme_kit::text;

/// Where a rollback window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackWindowStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollbackWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollbackWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback window as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackWindow {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub scope: String,
    pub created_at: i64,
    pub burst: i64,
    pub attempts: i64,
    pub limit: i64,
    pub margin: f64,
    pub audience: String,
    pub status: RollbackWindowStatus,
}

impl RollbackWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackWindowStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackWindowStatus) -> bool {
        match self.status {
            RollbackWindowStatus::Requested => {
                return next == RollbackWindowStatus::Granted;
            }
            RollbackWindowStatus::Granted => {
                return next == RollbackWindowStatus::Revoked || next == RollbackWindowStatus::Expired;
            }
            RollbackWindowStatus::Revoked => {
                return next == RollbackWindowStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollback window", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ends_at as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackWindow {
        RollbackWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            scope: self.scope.clone(),
            created_at: self.created_at,
            burst: self.burst,
            attempts: self.attempts,
            limit: self.limit,
            margin: self.margin,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rollback window in status requested.
pub fn new_rollback_window(code: &str, label: &str) -> RollbackWindow {
    RollbackWindow {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        scope: "".to_string(),
        created_at: 0,
        burst: 0,
        attempts: 0,
        limit: 0,
        margin: 0.0,
        audience: "".to_string(),
        status: RollbackWindowStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollback_window("code-426", "label-614");
        assert_eq!(item.status, RollbackWindowStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rollback_window("code-426", "label-614");
        if item.move_to(RollbackWindowStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackWindowStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollback_window("code-426", "label-614");
        item.status = RollbackWindowStatus::Expired;
        assert!(
            item.move_to(RollbackWindowStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
