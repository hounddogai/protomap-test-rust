use super::*;

use acme_kit::text;

/// Where a segment proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentProposalStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SegmentProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SegmentProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment proposal as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentProposal {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub rate: f64,
    pub priority: i64,
    pub timezone: String,
    pub batch_size: i64,
    pub cipher: String,
    pub scope: String,
    pub weight: f64,
    pub status: SegmentProposalStatus,
}

impl SegmentProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentProposalStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentProposalStatus) -> bool {
        match self.status {
            SegmentProposalStatus::Proposed => {
                return next == SegmentProposalStatus::Withdrawn;
            }
            SegmentProposalStatus::Approved => {
                return next == SegmentProposalStatus::Withdrawn;
            }
            SegmentProposalStatus::Rejected => {
                return next == SegmentProposalStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        (self.rate * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentProposal {
        SegmentProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            rate: self.rate,
            priority: self.priority,
            timezone: self.timezone.clone(),
            batch_size: self.batch_size,
            cipher: self.cipher.clone(),
            scope: self.scope.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a segment proposal in status proposed.
pub fn new_segment_proposal(code: &str, title: &str) -> SegmentProposal {
    SegmentProposal {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        rate: 0.0,
        priority: 0,
        timezone: "".to_string(),
        batch_size: 0,
        cipher: "".to_string(),
        scope: "".to_string(),
        weight: 0.0,
        status: SegmentProposalStatus::Proposed,
    }
}
