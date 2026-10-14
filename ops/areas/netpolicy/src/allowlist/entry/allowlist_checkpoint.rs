use super::*;

use acme_kit::text;

/// Where a allowlist checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AllowlistCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AllowlistCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCheckpoint {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub priority: i64,
    pub category: String,
    pub ends_at: i64,
    pub ratio: f64,
    pub discount: f64,
    pub external_ref: String,
    pub status: AllowlistCheckpointStatus,
}

impl AllowlistCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistCheckpointStatus) -> bool {
        match self.status {
            AllowlistCheckpointStatus::Requested => {
                return next == AllowlistCheckpointStatus::Revoked;
            }
            AllowlistCheckpointStatus::Granted => {
                match next {
                    AllowlistCheckpointStatus::Revoked | AllowlistCheckpointStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistCheckpointStatus::Revoked => {
                match next {
                    AllowlistCheckpointStatus::Expired => {
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

    pub fn move_to(&mut self, next: AllowlistCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist checkpoint {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.weight - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistCheckpoint {
        AllowlistCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            priority: self.priority,
            category: self.category.clone(),
            ends_at: self.ends_at,
            ratio: self.ratio,
            discount: self.discount,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a allowlist checkpoint in status requested.
pub fn new_allowlist_checkpoint(code: &str, name: &str) -> AllowlistCheckpoint {
    AllowlistCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        priority: 0,
        category: "".to_string(),
        ends_at: 0,
        ratio: 0.0,
        discount: 0.0,
        external_ref: "".to_string(),
        status: AllowlistCheckpointStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_allowlist_checkpoint("code-724", "name-125");
        assert_eq!(item.status, AllowlistCheckpointStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_allowlist_checkpoint("code-724", "name-125");
        if item.move_to(AllowlistCheckpointStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistCheckpointStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_allowlist_checkpoint("code-724", "name-125");
        item.status = AllowlistCheckpointStatus::Expired;
        assert!(
            item.move_to(AllowlistCheckpointStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
