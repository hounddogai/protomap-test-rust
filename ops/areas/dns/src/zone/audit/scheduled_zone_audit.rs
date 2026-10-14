use super::*;

use acme_kit::text;

/// Where a scheduled zone audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledZoneAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledZoneAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ScheduledZoneAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled zone audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneAudit {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub duration_minutes: i64,
    pub expires_at: i64,
    pub namespace: String,
    pub limit: i64,
    pub contact_email: String,
    pub floor_value: f64,
    pub tags: Vec<String>,
    pub archived: bool,
    pub status: ScheduledZoneAuditStatus,
}

impl ScheduledZoneAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledZoneAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledZoneAuditStatus) -> bool {
        match self.status {
            ScheduledZoneAuditStatus::Draft => {
                match next {
                    ScheduledZoneAuditStatus::Pending | ScheduledZoneAuditStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledZoneAuditStatus::Pending => {
                return next == ScheduledZoneAuditStatus::Suspended || next == ScheduledZoneAuditStatus::Closed;
            }
            ScheduledZoneAuditStatus::Active => {
                return next == ScheduledZoneAuditStatus::Closed;
            }
            ScheduledZoneAuditStatus::Suspended => {
                match next {
                    ScheduledZoneAuditStatus::Closed => {
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

    pub fn move_to(&mut self, next: ScheduledZoneAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled zone audit {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledZoneAudit {
        ScheduledZoneAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            duration_minutes: self.duration_minutes,
            expires_at: self.expires_at,
            namespace: self.namespace.clone(),
            limit: self.limit,
            contact_email: self.contact_email.clone(),
            floor_value: self.floor_value,
            tags: self.tags.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a scheduled zone audit in status draft.
pub fn new_scheduled_zone_audit(code: &str, label: &str) -> ScheduledZoneAudit {
    ScheduledZoneAudit {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        duration_minutes: 0,
        expires_at: 0,
        namespace: "".to_string(),
        limit: 0,
        contact_email: "".to_string(),
        floor_value: 0.0,
        tags: Vec::new(),
        archived: false,
        status: ScheduledZoneAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_zone_audit("code-338", "label-939");
        assert_eq!(item.status, ScheduledZoneAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_scheduled_zone_audit("code-338", "label-939");
        if item.move_to(ScheduledZoneAuditStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledZoneAuditStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_zone_audit("code-338", "label-939");
        item.status = ScheduledZoneAuditStatus::Closed;
        assert!(
            item.move_to(ScheduledZoneAuditStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
