use super::*;

use acme_kit::text;

/// Where a pending query audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingQueryAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingQueryAuditStatus {
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

impl std::fmt::Display for PendingQueryAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending query audit as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryAudit {
    pub code: String,
    pub name: String,
    pub country: String,
    pub cipher: String,
    pub discount: f64,
    pub status: PendingQueryAuditStatus,
}

impl PendingQueryAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingQueryAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingQueryAuditStatus) -> bool {
        match self.status {
            PendingQueryAuditStatus::Draft => {
                return next == PendingQueryAuditStatus::Active || next == PendingQueryAuditStatus::Closed;
            }
            PendingQueryAuditStatus::Pending => {
                return next == PendingQueryAuditStatus::Active;
            }
            PendingQueryAuditStatus::Active => {
                return next == PendingQueryAuditStatus::Suspended || next == PendingQueryAuditStatus::Closed;
            }
            PendingQueryAuditStatus::Suspended => {
                return next == PendingQueryAuditStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingQueryAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending query audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending query audit", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> PendingQueryAudit {
        PendingQueryAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            cipher: self.cipher.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a pending query audit in status draft.
pub fn new_pending_query_audit(code: &str, name: &str) -> PendingQueryAudit {
    PendingQueryAudit {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        cipher: "".to_string(),
        discount: 0.0,
        status: PendingQueryAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_query_audit("code-912", "name-589");
        assert_eq!(item.status, PendingQueryAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_pending_query_audit("code-912", "name-589");
        if item.move_to(PendingQueryAuditStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingQueryAuditStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_query_audit("code-912", "name-589");
        item.status = PendingQueryAuditStatus::Closed;
        assert!(
            item.move_to(PendingQueryAuditStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
