use super::*;

use acme_kit::text;

/// Where a pending audit event profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingAuditEventProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingAuditEventProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingAuditEventProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending audit event profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventProfile {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub issuer: String,
    pub phone: String,
    pub region: String,
    pub expires_at: i64,
    pub ttl_seconds: i64,
    pub floor_value: f64,
    pub name: String,
    pub currency: String,
    pub status: PendingAuditEventProfileStatus,
}

impl PendingAuditEventProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingAuditEventProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingAuditEventProfileStatus) -> bool {
        match self.status {
            PendingAuditEventProfileStatus::Requested => {
                match next {
                    PendingAuditEventProfileStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingAuditEventProfileStatus::Granted => {
                return next == PendingAuditEventProfileStatus::Revoked
                    || next == PendingAuditEventProfileStatus::Expired;
            }
            PendingAuditEventProfileStatus::Revoked => {
                match next {
                    PendingAuditEventProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: PendingAuditEventProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending audit event profile {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingAuditEventProfile {
        PendingAuditEventProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            issuer: self.issuer.clone(),
            phone: self.phone.clone(),
            region: self.region.clone(),
            expires_at: self.expires_at,
            ttl_seconds: self.ttl_seconds,
            floor_value: self.floor_value,
            name: self.name.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a pending audit event profile in status requested.
pub fn new_pending_audit_event_profile(code: &str, title: &str) -> PendingAuditEventProfile {
    PendingAuditEventProfile {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        issuer: "".to_string(),
        phone: "".to_string(),
        region: "".to_string(),
        expires_at: 0,
        ttl_seconds: 0,
        floor_value: 0.0,
        name: "".to_string(),
        currency: "".to_string(),
        status: PendingAuditEventProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_audit_event_profile("code-726", "title-915");
        assert_eq!(item.status, PendingAuditEventProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_pending_audit_event_profile("code-726", "title-915");
        if item.move_to(PendingAuditEventProfileStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingAuditEventProfileStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_audit_event_profile("code-726", "title-915");
        item.status = PendingAuditEventProfileStatus::Expired;
        assert!(
            item.move_to(PendingAuditEventProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
