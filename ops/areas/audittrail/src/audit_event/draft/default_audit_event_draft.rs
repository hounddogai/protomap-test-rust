use super::*;

use acme_kit::text;

/// Where a default audit event draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultAuditEventDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultAuditEventDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultAuditEventDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default audit event draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventDraft {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub expires_at: i64,
    pub category: String,
    pub key_id: String,
    pub rate: f64,
    pub reviewer: String,
    pub status: DefaultAuditEventDraftStatus,
}

impl DefaultAuditEventDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultAuditEventDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultAuditEventDraftStatus) -> bool {
        match self.status {
            DefaultAuditEventDraftStatus::Requested => {
                match next {
                    DefaultAuditEventDraftStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultAuditEventDraftStatus::Granted => {
                return next == DefaultAuditEventDraftStatus::Revoked || next == DefaultAuditEventDraftStatus::Expired;
            }
            DefaultAuditEventDraftStatus::Revoked => {
                return next == DefaultAuditEventDraftStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultAuditEventDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default audit event draft {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.expires_at as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultAuditEventDraft {
        DefaultAuditEventDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            expires_at: self.expires_at,
            category: self.category.clone(),
            key_id: self.key_id.clone(),
            rate: self.rate,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a default audit event draft in status requested.
pub fn new_default_audit_event_draft(code: &str, label: &str) -> DefaultAuditEventDraft {
    DefaultAuditEventDraft {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        expires_at: 0,
        category: "".to_string(),
        key_id: "".to_string(),
        rate: 0.0,
        reviewer: "".to_string(),
        status: DefaultAuditEventDraftStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_audit_event_draft("code-748", "label-63");
        assert_eq!(item.status, DefaultAuditEventDraftStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_default_audit_event_draft("code-748", "label-63");
        if item.move_to(DefaultAuditEventDraftStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultAuditEventDraftStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_audit_event_draft("code-748", "label-63");
        item.status = DefaultAuditEventDraftStatus::Expired;
        assert!(
            item.move_to(DefaultAuditEventDraftStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
