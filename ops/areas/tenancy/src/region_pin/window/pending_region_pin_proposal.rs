use super::*;

use acme_kit::text;

/// Where a pending region pin proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRegionPinProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingRegionPinProposalStatus {
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

impl std::fmt::Display for PendingRegionPinProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending region pin proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinProposal {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub score: f64,
    pub duration_minutes: i64,
    pub segment: String,
    pub version: i64,
    pub margin: f64,
    pub weight: f64,
    pub budget: f64,
    pub status: PendingRegionPinProposalStatus,
}

impl PendingRegionPinProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRegionPinProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRegionPinProposalStatus) -> bool {
        match self.status {
            PendingRegionPinProposalStatus::Planned => {
                match next {
                    PendingRegionPinProposalStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRegionPinProposalStatus::Scheduled => {
                return next == PendingRegionPinProposalStatus::Completed
                    || next == PendingRegionPinProposalStatus::Cancelled;
            }
            PendingRegionPinProposalStatus::InProgress => {
                return next == PendingRegionPinProposalStatus::Completed
                    || next == PendingRegionPinProposalStatus::Cancelled;
            }
            PendingRegionPinProposalStatus::Completed => {
                return next == PendingRegionPinProposalStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRegionPinProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending region pin proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRegionPinProposal {
        PendingRegionPinProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            score: self.score,
            duration_minutes: self.duration_minutes,
            segment: self.segment.clone(),
            version: self.version,
            margin: self.margin,
            weight: self.weight,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a pending region pin proposal in status planned.
pub fn new_pending_region_pin_proposal(code: &str, label: &str) -> PendingRegionPinProposal {
    PendingRegionPinProposal {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        score: 0.0,
        duration_minutes: 0,
        segment: "".to_string(),
        version: 0,
        margin: 0.0,
        weight: 0.0,
        budget: 0.0,
        status: PendingRegionPinProposalStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_region_pin_proposal("code-961", "label-799");
        assert_eq!(item.status, PendingRegionPinProposalStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_pending_region_pin_proposal("code-961", "label-799");
        if item.move_to(PendingRegionPinProposalStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRegionPinProposalStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_region_pin_proposal("code-961", "label-799");
        item.status = PendingRegionPinProposalStatus::Cancelled;
        assert!(
            item.move_to(PendingRegionPinProposalStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
