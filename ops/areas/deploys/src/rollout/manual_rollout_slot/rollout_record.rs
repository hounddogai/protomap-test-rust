use super::*;

use acme_kit::text;

/// Where a rollout record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutRecordStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RolloutRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RolloutRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutRecord {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub margin: f64,
    pub updated_at: i64,
    pub priority: i64,
    pub principal: String,
    pub quantity: i64,
    pub status: RolloutRecordStatus,
}

impl RolloutRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutRecordStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutRecordStatus) -> bool {
        match self.status {
            RolloutRecordStatus::Requested => {
                return next == RolloutRecordStatus::Revoked || next == RolloutRecordStatus::Expired;
            }
            RolloutRecordStatus::Granted => {
                return next == RolloutRecordStatus::Revoked || next == RolloutRecordStatus::Expired;
            }
            RolloutRecordStatus::Revoked => {
                match next {
                    RolloutRecordStatus::Expired => {
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

    pub fn move_to(&mut self, next: RolloutRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.updated_at as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutRecord {
        RolloutRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            margin: self.margin,
            updated_at: self.updated_at,
            priority: self.priority,
            principal: self.principal.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rollout record in status requested.
pub fn new_rollout_record(code: &str, title: &str) -> RolloutRecord {
    RolloutRecord {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        margin: 0.0,
        updated_at: 0,
        priority: 0,
        principal: "".to_string(),
        quantity: 0,
        status: RolloutRecordStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollout_record("code-621", "title-663");
        assert_eq!(item.status, RolloutRecordStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rollout_record("code-621", "title-663");
        if item.move_to(RolloutRecordStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutRecordStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollout_record("code-621", "title-663");
        item.status = RolloutRecordStatus::Expired;
        assert!(
            item.move_to(RolloutRecordStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
