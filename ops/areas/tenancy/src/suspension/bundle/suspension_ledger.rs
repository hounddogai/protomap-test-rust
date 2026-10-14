use super::*;

use acme_kit::text;

/// Where a suspension ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionLedgerStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SuspensionLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for SuspensionLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionLedger {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub phone: String,
    pub expires_at: i64,
    pub discount: f64,
    pub principal: String,
    pub priority: i64,
    pub visible: bool,
    pub status: SuspensionLedgerStatus,
}

impl SuspensionLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionLedgerStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionLedgerStatus) -> bool {
        match self.status {
            SuspensionLedgerStatus::Open => {
                match next {
                    SuspensionLedgerStatus::InReview | SuspensionLedgerStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionLedgerStatus::InReview => {
                return next == SuspensionLedgerStatus::Reopened || next == SuspensionLedgerStatus::Archived;
            }
            SuspensionLedgerStatus::Resolved => {
                match next {
                    SuspensionLedgerStatus::Reopened | SuspensionLedgerStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionLedgerStatus::Reopened => {
                return next == SuspensionLedgerStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "suspension ledger", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.expires_at as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionLedger {
        SuspensionLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            phone: self.phone.clone(),
            expires_at: self.expires_at,
            discount: self.discount,
            principal: self.principal.clone(),
            priority: self.priority,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a suspension ledger in status open.
pub fn new_suspension_ledger(code: &str, label: &str) -> SuspensionLedger {
    SuspensionLedger {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        phone: "".to_string(),
        expires_at: 0,
        discount: 0.0,
        principal: "".to_string(),
        priority: 0,
        visible: false,
        status: SuspensionLedgerStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_suspension_ledger("code-365", "label-56");
        assert_eq!(item.status, SuspensionLedgerStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_suspension_ledger("code-365", "label-56");
        if item.move_to(SuspensionLedgerStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionLedgerStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_suspension_ledger("code-365", "label-56");
        item.status = SuspensionLedgerStatus::Archived;
        assert!(
            item.move_to(SuspensionLedgerStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
