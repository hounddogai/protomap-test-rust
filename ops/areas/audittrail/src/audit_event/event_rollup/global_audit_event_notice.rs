use super::*;

use acme_kit::text;

/// Where a global audit event notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalAuditEventNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalAuditEventNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalAuditEventNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global audit event notice as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventNotice {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub weight: f64,
    pub discount: f64,
    pub status: GlobalAuditEventNoticeStatus,
}

impl GlobalAuditEventNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalAuditEventNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalAuditEventNoticeStatus) -> bool {
        match self.status {
            GlobalAuditEventNoticeStatus::Proposed => {
                match next {
                    GlobalAuditEventNoticeStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalAuditEventNoticeStatus::Approved => {
                return next == GlobalAuditEventNoticeStatus::Withdrawn;
            }
            GlobalAuditEventNoticeStatus::Rejected => {
                match next {
                    GlobalAuditEventNoticeStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: GlobalAuditEventNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global audit event notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global audit event notice", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.discount - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalAuditEventNotice {
        GlobalAuditEventNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            weight: self.weight,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global audit event notice in status proposed.
pub fn new_global_audit_event_notice(code: &str, title: &str) -> GlobalAuditEventNotice {
    GlobalAuditEventNotice {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        weight: 0.0,
        discount: 0.0,
        status: GlobalAuditEventNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_global_audit_event_notice("code-948", "title-819");
        assert_eq!(item.status, GlobalAuditEventNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_global_audit_event_notice("code-948", "title-819");
        if item.move_to(GlobalAuditEventNoticeStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalAuditEventNoticeStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_global_audit_event_notice("code-948", "title-819");
        item.status = GlobalAuditEventNoticeStatus::Withdrawn;
        assert!(
            item.move_to(GlobalAuditEventNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
