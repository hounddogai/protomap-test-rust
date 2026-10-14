use super::*;

use acme_kit::text;

/// Where a rolling segment proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSegmentProposalStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingSegmentProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingSegmentProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling segment proposal as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentProposal {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub rate: f64,
    pub source: String,
    pub version: i64,
    pub ceiling: f64,
    pub attempts: i64,
    pub external_ref: String,
    pub duration_minutes: i64,
    pub segment: String,
    pub status: RollingSegmentProposalStatus,
}

impl RollingSegmentProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSegmentProposalStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSegmentProposalStatus) -> bool {
        match self.status {
            RollingSegmentProposalStatus::Proposed => {
                match next {
                    RollingSegmentProposalStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSegmentProposalStatus::Approved => {
                return next == RollingSegmentProposalStatus::Withdrawn;
            }
            RollingSegmentProposalStatus::Rejected => {
                return next == RollingSegmentProposalStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingSegmentProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling segment proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ceiling - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingSegmentProposal {
        RollingSegmentProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            rate: self.rate,
            source: self.source.clone(),
            version: self.version,
            ceiling: self.ceiling,
            attempts: self.attempts,
            external_ref: self.external_ref.clone(),
            duration_minutes: self.duration_minutes,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling segment proposal in status proposed.
pub fn new_rolling_segment_proposal(code: &str, name: &str) -> RollingSegmentProposal {
    RollingSegmentProposal {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        rate: 0.0,
        source: "".to_string(),
        version: 0,
        ceiling: 0.0,
        attempts: 0,
        external_ref: "".to_string(),
        duration_minutes: 0,
        segment: "".to_string(),
        status: RollingSegmentProposalStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_segment_proposal("code-544", "name-516");
        assert_eq!(item.status, RollingSegmentProposalStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_segment_proposal("code-544", "name-516");
        if item.move_to(RollingSegmentProposalStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingSegmentProposalStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_segment_proposal("code-544", "name-516");
        item.status = RollingSegmentProposalStatus::Withdrawn;
        assert!(
            item.move_to(RollingSegmentProposalStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
