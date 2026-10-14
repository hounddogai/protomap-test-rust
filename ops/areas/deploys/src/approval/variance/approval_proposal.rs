use super::*;

use acme_kit::text;

/// Where a approval proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalProposalStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ApprovalProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ApprovalProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a approval proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalProposal {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub active: bool,
    pub approved: bool,
    pub batch_size: i64,
    pub window_days: i64,
    pub unit: String,
    pub ceiling: f64,
    pub audience: String,
    pub checksum: String,
    pub status: ApprovalProposalStatus,
}

impl ApprovalProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalProposalStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalProposalStatus) -> bool {
        match self.status {
            ApprovalProposalStatus::Idle => {
                return next == ApprovalProposalStatus::Warming || next == ApprovalProposalStatus::Stopped;
            }
            ApprovalProposalStatus::Warming => {
                match next {
                    ApprovalProposalStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalProposalStatus::Ready => {
                return next == ApprovalProposalStatus::Stopped;
            }
            ApprovalProposalStatus::Draining => {
                match next {
                    ApprovalProposalStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ApprovalProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "approval proposal", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ceiling + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalProposal {
        ApprovalProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            active: self.active,
            approved: self.approved,
            batch_size: self.batch_size,
            window_days: self.window_days,
            unit: self.unit.clone(),
            ceiling: self.ceiling,
            audience: self.audience.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a approval proposal in status idle.
pub fn new_approval_proposal(code: &str, label: &str) -> ApprovalProposal {
    ApprovalProposal {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        active: false,
        approved: false,
        batch_size: 0,
        window_days: 0,
        unit: "".to_string(),
        ceiling: 0.0,
        audience: "".to_string(),
        checksum: "".to_string(),
        status: ApprovalProposalStatus::Idle,
    }
}
