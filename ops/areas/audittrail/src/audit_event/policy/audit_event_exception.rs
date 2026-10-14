use super::*;

use acme_kit::text;

/// Where a audit event exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventExceptionStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AuditEventExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AuditEventExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a audit event exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventException {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub expires_at: i64,
    pub name: String,
    pub ratio: f64,
    pub grace_days: i64,
    pub cipher: String,
    pub port: i64,
    pub cost: f64,
    pub discount: f64,
    pub status: AuditEventExceptionStatus,
}

impl AuditEventException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventExceptionStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventExceptionStatus) -> bool {
        match self.status {
            AuditEventExceptionStatus::Requested => {
                return next == AuditEventExceptionStatus::Revoked || next == AuditEventExceptionStatus::Expired;
            }
            AuditEventExceptionStatus::Granted => {
                match next {
                    AuditEventExceptionStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventExceptionStatus::Revoked => {
                match next {
                    AuditEventExceptionStatus::Expired => {
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

    pub fn move_to(&mut self, next: AuditEventExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "audit event exception", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.discount - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventException {
        AuditEventException {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            expires_at: self.expires_at,
            name: self.name.clone(),
            ratio: self.ratio,
            grace_days: self.grace_days,
            cipher: self.cipher.clone(),
            port: self.port,
            cost: self.cost,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a audit event exception in status requested.
pub fn new_audit_event_exception(code: &str, title: &str) -> AuditEventException {
    AuditEventException {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        expires_at: 0,
        name: "".to_string(),
        ratio: 0.0,
        grace_days: 0,
        cipher: "".to_string(),
        port: 0,
        cost: 0.0,
        discount: 0.0,
        status: AuditEventExceptionStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_audit_event_exception("code-532", "title-366");
        assert_eq!(item.status, AuditEventExceptionStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_audit_event_exception("code-532", "title-366");
        if item.move_to(AuditEventExceptionStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventExceptionStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_audit_event_exception("code-532", "title-366");
        item.status = AuditEventExceptionStatus::Expired;
        assert!(
            item.move_to(AuditEventExceptionStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
