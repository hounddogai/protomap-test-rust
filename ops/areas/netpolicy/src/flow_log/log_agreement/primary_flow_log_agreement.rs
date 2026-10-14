use super::*;

use acme_kit::text;

/// Where a primary flow log agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryFlowLogAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryFlowLogAgreementStatus {
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

impl std::fmt::Display for PrimaryFlowLogAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary flow log agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFlowLogAgreement {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub key_id: String,
    pub source: String,
    pub quantity: i64,
    pub status: PrimaryFlowLogAgreementStatus,
}

impl PrimaryFlowLogAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryFlowLogAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryFlowLogAgreementStatus) -> bool {
        match self.status {
            PrimaryFlowLogAgreementStatus::Draft => {
                return next == PrimaryFlowLogAgreementStatus::Active;
            }
            PrimaryFlowLogAgreementStatus::Pending => {
                return next == PrimaryFlowLogAgreementStatus::Active;
            }
            PrimaryFlowLogAgreementStatus::Active => {
                return next == PrimaryFlowLogAgreementStatus::Closed;
            }
            PrimaryFlowLogAgreementStatus::Suspended => {
                match next {
                    PrimaryFlowLogAgreementStatus::Closed => {
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

    pub fn move_to(&mut self, next: PrimaryFlowLogAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary flow log agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> PrimaryFlowLogAgreement {
        PrimaryFlowLogAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            key_id: self.key_id.clone(),
            source: self.source.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary flow log agreement in status draft.
pub fn new_primary_flow_log_agreement(code: &str, name: &str) -> PrimaryFlowLogAgreement {
    PrimaryFlowLogAgreement {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        key_id: "".to_string(),
        source: "".to_string(),
        quantity: 0,
        status: PrimaryFlowLogAgreementStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_flow_log_agreement("code-95", "name-9");
        assert_eq!(item.status, PrimaryFlowLogAgreementStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_primary_flow_log_agreement("code-95", "name-9");
        if item.move_to(PrimaryFlowLogAgreementStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryFlowLogAgreementStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_flow_log_agreement("code-95", "name-9");
        item.status = PrimaryFlowLogAgreementStatus::Closed;
        assert!(
            item.move_to(PrimaryFlowLogAgreementStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
