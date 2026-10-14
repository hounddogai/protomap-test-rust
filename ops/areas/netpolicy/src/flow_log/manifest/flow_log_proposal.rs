use super::*;

use acme_kit::text;

/// Where a flow log proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FlowLogProposalStatus {
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

impl std::fmt::Display for FlowLogProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log proposal as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProposal {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub host: String,
    pub fingerprint: String,
    pub timezone: String,
    pub external_ref: String,
    pub source: String,
    pub active: bool,
    pub segment: String,
    pub rotation_days: i64,
    pub status: FlowLogProposalStatus,
}

impl FlowLogProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogProposalStatus) -> bool {
        match self.status {
            FlowLogProposalStatus::Draft => {
                match next {
                    FlowLogProposalStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogProposalStatus::Pending => {
                return next == FlowLogProposalStatus::Suspended || next == FlowLogProposalStatus::Closed;
            }
            FlowLogProposalStatus::Active => {
                match next {
                    FlowLogProposalStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogProposalStatus::Suspended => {
                match next {
                    FlowLogProposalStatus::Closed => {
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

    pub fn move_to(&mut self, next: FlowLogProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> FlowLogProposal {
        FlowLogProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            host: self.host.clone(),
            fingerprint: self.fingerprint.clone(),
            timezone: self.timezone.clone(),
            external_ref: self.external_ref.clone(),
            source: self.source.clone(),
            active: self.active,
            segment: self.segment.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a flow log proposal in status draft.
pub fn new_flow_log_proposal(code: &str, label: &str) -> FlowLogProposal {
    FlowLogProposal {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        host: "".to_string(),
        fingerprint: "".to_string(),
        timezone: "".to_string(),
        external_ref: "".to_string(),
        source: "".to_string(),
        active: false,
        segment: "".to_string(),
        rotation_days: 0,
        status: FlowLogProposalStatus::Draft,
    }
}
