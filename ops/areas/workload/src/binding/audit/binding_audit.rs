use super::*;

use acme_kit::text;

/// Where a binding audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BindingAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BindingAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding audit as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingAudit {
    pub code: String,
    pub title: String,
    pub region: String,
    pub notes: String,
    pub duration_minutes: i64,
    pub status: BindingAuditStatus,
}

impl BindingAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingAuditStatus) -> bool {
        match self.status {
            BindingAuditStatus::Requested => {
                return next == BindingAuditStatus::Expired;
            }
            BindingAuditStatus::Granted => {
                return next == BindingAuditStatus::Revoked;
            }
            BindingAuditStatus::Revoked => {
                return next == BindingAuditStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> BindingAudit {
        BindingAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            notes: self.notes.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a binding audit in status requested.
pub fn new_binding_audit(code: &str, title: &str) -> BindingAudit {
    BindingAudit {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        notes: "".to_string(),
        duration_minutes: 0,
        status: BindingAuditStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_binding_audit("code-753", "title-455");
        assert_eq!(item.status, BindingAuditStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_binding_audit("code-753", "title-455");
        if item.move_to(BindingAuditStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingAuditStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_binding_audit("code-753", "title-455");
        item.status = BindingAuditStatus::Expired;
        assert!(
            item.move_to(BindingAuditStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
