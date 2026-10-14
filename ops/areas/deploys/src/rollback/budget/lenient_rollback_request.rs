use super::*;

use acme_kit::text;

/// Where a lenient rollback request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRollbackRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientRollbackRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientRollbackRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient rollback request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRollbackRequest {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub issuer: String,
    pub archived: bool,
    pub quantity: i64,
    pub quota_units: i64,
    pub created_at: i64,
    pub expires_at: i64,
    pub priority: i64,
    pub status: LenientRollbackRequestStatus,
}

impl LenientRollbackRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRollbackRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRollbackRequestStatus) -> bool {
        match self.status {
            LenientRollbackRequestStatus::Requested => {
                return next == LenientRollbackRequestStatus::Granted || next == LenientRollbackRequestStatus::Revoked;
            }
            LenientRollbackRequestStatus::Granted => {
                return next == LenientRollbackRequestStatus::Revoked || next == LenientRollbackRequestStatus::Expired;
            }
            LenientRollbackRequestStatus::Revoked => {
                match next {
                    LenientRollbackRequestStatus::Expired => {
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

    pub fn move_to(&mut self, next: LenientRollbackRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient rollback request {} cannot move from {} to {}",
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.quota_units as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRollbackRequest {
        LenientRollbackRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            issuer: self.issuer.clone(),
            archived: self.archived,
            quantity: self.quantity,
            quota_units: self.quota_units,
            created_at: self.created_at,
            expires_at: self.expires_at,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient rollback request in status requested.
pub fn new_lenient_rollback_request(code: &str, name: &str) -> LenientRollbackRequest {
    LenientRollbackRequest {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        issuer: "".to_string(),
        archived: false,
        quantity: 0,
        quota_units: 0,
        created_at: 0,
        expires_at: 0,
        priority: 0,
        status: LenientRollbackRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_rollback_request("code-412", "name-271");
        assert_eq!(item.status, LenientRollbackRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_lenient_rollback_request("code-412", "name-271");
        if item.move_to(LenientRollbackRequestStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRollbackRequestStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_rollback_request("code-412", "name-271");
        item.status = LenientRollbackRequestStatus::Expired;
        assert!(
            item.move_to(LenientRollbackRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
