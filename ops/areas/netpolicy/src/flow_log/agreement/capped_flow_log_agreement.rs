use super::*;

use acme_kit::text;

/// Where a capped flow log agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedFlowLogAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedFlowLogAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for CappedFlowLogAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped flow log agreement as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFlowLogAgreement {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub active: bool,
    pub capacity: i64,
    pub grace_days: i64,
    pub external_ref: String,
    pub scope: String,
    pub reviewer: String,
    pub contact_email: String,
    pub rate: f64,
    pub status: CappedFlowLogAgreementStatus,
}

impl CappedFlowLogAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedFlowLogAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedFlowLogAgreementStatus) -> bool {
        match self.status {
            CappedFlowLogAgreementStatus::New => {
                match next {
                    CappedFlowLogAgreementStatus::Queued | CappedFlowLogAgreementStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFlowLogAgreementStatus::Queued => {
                return next == CappedFlowLogAgreementStatus::Running || next == CappedFlowLogAgreementStatus::Failed;
            }
            CappedFlowLogAgreementStatus::Running => {
                match next {
                    CappedFlowLogAgreementStatus::Done | CappedFlowLogAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFlowLogAgreementStatus::Done => {
                match next {
                    CappedFlowLogAgreementStatus::Failed => {
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

    pub fn move_to(&mut self, next: CappedFlowLogAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped flow log agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped flow log agreement", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.grace_days as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedFlowLogAgreement {
        CappedFlowLogAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            active: self.active,
            capacity: self.capacity,
            grace_days: self.grace_days,
            external_ref: self.external_ref.clone(),
            scope: self.scope.clone(),
            reviewer: self.reviewer.clone(),
            contact_email: self.contact_email.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a capped flow log agreement in status new.
pub fn new_capped_flow_log_agreement(code: &str, label: &str) -> CappedFlowLogAgreement {
    CappedFlowLogAgreement {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        active: false,
        capacity: 0,
        grace_days: 0,
        external_ref: "".to_string(),
        scope: "".to_string(),
        reviewer: "".to_string(),
        contact_email: "".to_string(),
        rate: 0.0,
        status: CappedFlowLogAgreementStatus::New,
    }
}
