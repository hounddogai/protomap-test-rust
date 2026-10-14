use super::*;

use acme_kit::text;

/// Where a global timeline proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalTimelineProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalTimelineProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for GlobalTimelineProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global timeline proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineProposal {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub discount: f64,
    pub unit: String,
    pub tags: Vec<String>,
    pub source: String,
    pub status: GlobalTimelineProposalStatus,
}

impl GlobalTimelineProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalTimelineProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalTimelineProposalStatus) -> bool {
        match self.status {
            GlobalTimelineProposalStatus::Planned => {
                return next == GlobalTimelineProposalStatus::Cancelled;
            }
            GlobalTimelineProposalStatus::Scheduled => {
                match next {
                    GlobalTimelineProposalStatus::InProgress | GlobalTimelineProposalStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTimelineProposalStatus::InProgress => {
                return next == GlobalTimelineProposalStatus::Completed
                    || next == GlobalTimelineProposalStatus::Cancelled;
            }
            GlobalTimelineProposalStatus::Completed => {
                return next == GlobalTimelineProposalStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalTimelineProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global timeline proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rate - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalTimelineProposal {
        GlobalTimelineProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            discount: self.discount,
            unit: self.unit.clone(),
            tags: self.tags.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a global timeline proposal in status planned.
pub fn new_global_timeline_proposal(code: &str, name: &str) -> GlobalTimelineProposal {
    GlobalTimelineProposal {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        discount: 0.0,
        unit: "".to_string(),
        tags: Vec::new(),
        source: "".to_string(),
        status: GlobalTimelineProposalStatus::Planned,
    }
}
