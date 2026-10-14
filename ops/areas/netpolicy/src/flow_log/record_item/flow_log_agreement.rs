use super::*;

use acme_kit::text;

/// Where a flow log agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FlowLogAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FlowLogAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log agreement as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAgreement {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub namespace: String,
    pub key_id: String,
    pub segment: String,
    pub limit: i64,
    pub ratio: f64,
    pub status: FlowLogAgreementStatus,
}

impl FlowLogAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogAgreementStatus) -> bool {
        match self.status {
            FlowLogAgreementStatus::Requested => {
                return next == FlowLogAgreementStatus::Granted || next == FlowLogAgreementStatus::Expired;
            }
            FlowLogAgreementStatus::Granted => {
                match next {
                    FlowLogAgreementStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogAgreementStatus::Revoked => {
                match next {
                    FlowLogAgreementStatus::Expired => {
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

    pub fn move_to(&mut self, next: FlowLogAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "flow log agreement", self.code)
    }

    pub fn net_total(&self) -> f64 {
        (self.ratio * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogAgreement {
        FlowLogAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            namespace: self.namespace.clone(),
            key_id: self.key_id.clone(),
            segment: self.segment.clone(),
            limit: self.limit,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a flow log agreement in status requested.
pub fn new_flow_log_agreement(code: &str, label: &str) -> FlowLogAgreement {
    FlowLogAgreement {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        namespace: "".to_string(),
        key_id: "".to_string(),
        segment: "".to_string(),
        limit: 0,
        ratio: 0.0,
        status: FlowLogAgreementStatus::Requested,
    }
}
