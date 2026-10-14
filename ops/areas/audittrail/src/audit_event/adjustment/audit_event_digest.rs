use super::*;

use acme_kit::text;

/// Where a audit event digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventDigestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AuditEventDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AuditEventDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event digest as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventDigest {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub external_ref: String,
    pub burst: i64,
    pub ends_at: i64,
    pub country: String,
    pub reviewer: String,
    pub status: AuditEventDigestStatus,
}

impl AuditEventDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventDigestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventDigestStatus) -> bool {
        match self.status {
            AuditEventDigestStatus::Created => {
                return next == AuditEventDigestStatus::Retired;
            }
            AuditEventDigestStatus::Validated => {
                match next {
                    AuditEventDigestStatus::Published | AuditEventDigestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventDigestStatus::Published => {
                match next {
                    AuditEventDigestStatus::Retired => {
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

    pub fn move_to(&mut self, next: AuditEventDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event digest {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ends_at as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventDigest {
        AuditEventDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            external_ref: self.external_ref.clone(),
            burst: self.burst,
            ends_at: self.ends_at,
            country: self.country.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a audit event digest in status created.
pub fn new_audit_event_digest(code: &str, title: &str) -> AuditEventDigest {
    AuditEventDigest {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        external_ref: "".to_string(),
        burst: 0,
        ends_at: 0,
        country: "".to_string(),
        reviewer: "".to_string(),
        status: AuditEventDigestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_audit_event_digest("code-848", "title-417");
        assert_eq!(item.status, AuditEventDigestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_audit_event_digest("code-848", "title-417");
        if item.move_to(AuditEventDigestStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventDigestStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_audit_event_digest("code-848", "title-417");
        item.status = AuditEventDigestStatus::Retired;
        assert!(
            item.move_to(AuditEventDigestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
