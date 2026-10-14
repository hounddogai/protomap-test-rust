use super::*;

use acme_kit::text;

/// Where a health check ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckLedgerStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl HealthCheckLedgerStatus {
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

impl std::fmt::Display for HealthCheckLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckLedger {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub approved: bool,
    pub visible: bool,
    pub source: String,
    pub contact_email: String,
    pub discount: f64,
    pub description: String,
    pub status: HealthCheckLedgerStatus,
}

impl HealthCheckLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckLedgerStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckLedgerStatus) -> bool {
        match self.status {
            HealthCheckLedgerStatus::Open => {
                match next {
                    HealthCheckLedgerStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckLedgerStatus::InReview => {
                return next == HealthCheckLedgerStatus::Resolved || next == HealthCheckLedgerStatus::Archived;
            }
            HealthCheckLedgerStatus::Resolved => {
                match next {
                    HealthCheckLedgerStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckLedgerStatus::Reopened => {
                match next {
                    HealthCheckLedgerStatus::Archived => {
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

    pub fn move_to(&mut self, next: HealthCheckLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> HealthCheckLedger {
        HealthCheckLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            approved: self.approved,
            visible: self.visible,
            source: self.source.clone(),
            contact_email: self.contact_email.clone(),
            discount: self.discount,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a health check ledger in status open.
pub fn new_health_check_ledger(code: &str, title: &str) -> HealthCheckLedger {
    HealthCheckLedger {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        approved: false,
        visible: false,
        source: "".to_string(),
        contact_email: "".to_string(),
        discount: 0.0,
        description: "".to_string(),
        status: HealthCheckLedgerStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_health_check_ledger("code-686", "title-938");
        assert_eq!(item.status, HealthCheckLedgerStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_health_check_ledger("code-686", "title-938");
        if item.move_to(HealthCheckLedgerStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckLedgerStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_health_check_ledger("code-686", "title-938");
        item.status = HealthCheckLedgerStatus::Archived;
        assert!(
            item.move_to(HealthCheckLedgerStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
