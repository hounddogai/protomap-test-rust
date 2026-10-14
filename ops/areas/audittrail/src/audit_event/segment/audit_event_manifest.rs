use super::*;

use acme_kit::text;

/// Where a audit event manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventManifestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AuditEventManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AuditEventManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One audit event manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventManifest {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub window_days: i64,
    pub grace_days: i64,
    pub namespace: String,
    pub currency: String,
    pub maximum: f64,
    pub starts_at: i64,
    pub rate: f64,
    pub status: AuditEventManifestStatus,
}

impl AuditEventManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventManifestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventManifestStatus) -> bool {
        match self.status {
            AuditEventManifestStatus::Proposed => {
                match next {
                    AuditEventManifestStatus::Approved | AuditEventManifestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventManifestStatus::Approved => {
                match next {
                    AuditEventManifestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventManifestStatus::Rejected => {
                match next {
                    AuditEventManifestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AuditEventManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        ((self.maximum + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventManifest {
        AuditEventManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            window_days: self.window_days,
            grace_days: self.grace_days,
            namespace: self.namespace.clone(),
            currency: self.currency.clone(),
            maximum: self.maximum,
            starts_at: self.starts_at,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a audit event manifest in status proposed.
pub fn new_audit_event_manifest(code: &str, title: &str) -> AuditEventManifest {
    AuditEventManifest {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        window_days: 0,
        grace_days: 0,
        namespace: "".to_string(),
        currency: "".to_string(),
        maximum: 0.0,
        starts_at: 0,
        rate: 0.0,
        status: AuditEventManifestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_audit_event_manifest("code-232", "title-611");
        assert_eq!(item.status, AuditEventManifestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_audit_event_manifest("code-232", "title-611");
        if item.move_to(AuditEventManifestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventManifestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_audit_event_manifest("code-232", "title-611");
        item.status = AuditEventManifestStatus::Withdrawn;
        assert!(
            item.move_to(AuditEventManifestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
