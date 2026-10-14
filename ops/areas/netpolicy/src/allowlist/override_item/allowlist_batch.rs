use super::*;

use acme_kit::text;

/// Where a allowlist batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AllowlistBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AllowlistBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBatch {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub reviewer: String,
    pub issuer: String,
    pub key_id: String,
    pub visible: bool,
    pub updated_at: i64,
    pub status: AllowlistBatchStatus,
}

impl AllowlistBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistBatchStatus) -> bool {
        match self.status {
            AllowlistBatchStatus::Requested => {
                return next == AllowlistBatchStatus::Granted;
            }
            AllowlistBatchStatus::Granted => {
                return next == AllowlistBatchStatus::Revoked || next == AllowlistBatchStatus::Expired;
            }
            AllowlistBatchStatus::Revoked => {
                return next == AllowlistBatchStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist batch {} cannot move from {} to {}",
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
        format!("{} {}", "allowlist batch", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> AllowlistBatch {
        AllowlistBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            reviewer: self.reviewer.clone(),
            issuer: self.issuer.clone(),
            key_id: self.key_id.clone(),
            visible: self.visible,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a allowlist batch in status requested.
pub fn new_allowlist_batch(code: &str, name: &str) -> AllowlistBatch {
    AllowlistBatch {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        reviewer: "".to_string(),
        issuer: "".to_string(),
        key_id: "".to_string(),
        visible: false,
        updated_at: 0,
        status: AllowlistBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_allowlist_batch("code-430", "name-730");
        assert_eq!(item.status, AllowlistBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_allowlist_batch("code-430", "name-730");
        if item.move_to(AllowlistBatchStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistBatchStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_allowlist_batch("code-430", "name-730");
        item.status = AllowlistBatchStatus::Expired;
        assert!(
            item.move_to(AllowlistBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
