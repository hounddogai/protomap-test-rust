use super::*;

use acme_kit::text;

/// Where a audit event milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventMilestoneStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AuditEventMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AuditEventMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a audit event milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMilestone {
    pub code: String,
    pub name: String,
    pub description: String,
    pub segment: String,
    pub quota_units: i64,
    pub title: String,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub checksum: String,
    pub key_id: String,
    pub batch_size: i64,
    pub status: AuditEventMilestoneStatus,
}

impl AuditEventMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventMilestoneStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventMilestoneStatus) -> bool {
        match self.status {
            AuditEventMilestoneStatus::Created => {
                match next {
                    AuditEventMilestoneStatus::Validated | AuditEventMilestoneStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventMilestoneStatus::Validated => {
                return next == AuditEventMilestoneStatus::Published || next == AuditEventMilestoneStatus::Retired;
            }
            AuditEventMilestoneStatus::Published => {
                return next == AuditEventMilestoneStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event milestone {} cannot move from {} to {}",
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
        format!("{} {}", "audit event milestone", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_total(&self) -> f64 {
        ((self.batch_size as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventMilestone {
        AuditEventMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            segment: self.segment.clone(),
            quota_units: self.quota_units,
            title: self.title.clone(),
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            checksum: self.checksum.clone(),
            key_id: self.key_id.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a audit event milestone in status created.
pub fn new_audit_event_milestone(code: &str, name: &str) -> AuditEventMilestone {
    AuditEventMilestone {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        segment: "".to_string(),
        quota_units: 0,
        title: "".to_string(),
        expires_at: 0,
        tags: Vec::new(),
        checksum: "".to_string(),
        key_id: "".to_string(),
        batch_size: 0,
        status: AuditEventMilestoneStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_audit_event_milestone("code-364", "name-122");
        assert_eq!(item.status, AuditEventMilestoneStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_audit_event_milestone("code-364", "name-122");
        if item.move_to(AuditEventMilestoneStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventMilestoneStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_audit_event_milestone("code-364", "name-122");
        item.status = AuditEventMilestoneStatus::Retired;
        assert!(
            item.move_to(AuditEventMilestoneStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
