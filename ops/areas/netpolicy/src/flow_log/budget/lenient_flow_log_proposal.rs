use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient flow log proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFlowLogProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientFlowLogProposalStatus {
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

impl std::fmt::Display for LenientFlowLogProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient flow log proposal as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFlowLogProposal {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub capacity: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: LenientFlowLogProposalStatus,
}

impl LenientFlowLogProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFlowLogProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFlowLogProposalStatus) -> bool {
        match self.status {
            LenientFlowLogProposalStatus::New => {
                match next {
                    LenientFlowLogProposalStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFlowLogProposalStatus::Queued => {
                match next {
                    LenientFlowLogProposalStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFlowLogProposalStatus::Running => {
                match next {
                    LenientFlowLogProposalStatus::Done | LenientFlowLogProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFlowLogProposalStatus::Done => {
                return next == LenientFlowLogProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientFlowLogProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient flow log proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> LenientFlowLogProposal {
        LenientFlowLogProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            capacity: self.capacity,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient flow log proposal in status new.
pub fn new_lenient_flow_log_proposal(code: &str, label: &str) -> LenientFlowLogProposal {
    LenientFlowLogProposal {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        capacity: 0,
        attributes: BTreeMap::new(),
        status: LenientFlowLogProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_flow_log_proposal("code-426", "label-967");
        assert_eq!(item.status, LenientFlowLogProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_lenient_flow_log_proposal("code-426", "label-967");
        if item.move_to(LenientFlowLogProposalStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientFlowLogProposalStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_flow_log_proposal("code-426", "label-967");
        item.status = LenientFlowLogProposalStatus::Failed;
        assert!(
            item.move_to(LenientFlowLogProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
