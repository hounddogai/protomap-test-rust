use super::*;

use acme_kit::text;

/// Where a legacy segment proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacySegmentProposalStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacySegmentProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LegacySegmentProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy segment proposal as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentProposal {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub checksum: String,
    pub summary: String,
    pub cluster: String,
    pub starts_at: i64,
    pub segment: String,
    pub audience: String,
    pub external_ref: String,
    pub status: LegacySegmentProposalStatus,
}

impl LegacySegmentProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacySegmentProposalStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacySegmentProposalStatus) -> bool {
        match self.status {
            LegacySegmentProposalStatus::Open => {
                return next == LegacySegmentProposalStatus::Resolved || next == LegacySegmentProposalStatus::Archived;
            }
            LegacySegmentProposalStatus::InReview => {
                match next {
                    LegacySegmentProposalStatus::Resolved | LegacySegmentProposalStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacySegmentProposalStatus::Resolved => {
                match next {
                    LegacySegmentProposalStatus::Reopened | LegacySegmentProposalStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacySegmentProposalStatus::Reopened => {
                return next == LegacySegmentProposalStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacySegmentProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy segment proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy segment proposal", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.starts_at as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacySegmentProposal {
        LegacySegmentProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            cluster: self.cluster.clone(),
            starts_at: self.starts_at,
            segment: self.segment.clone(),
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a legacy segment proposal in status open.
pub fn new_legacy_segment_proposal(code: &str, label: &str) -> LegacySegmentProposal {
    LegacySegmentProposal {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        checksum: "".to_string(),
        summary: "".to_string(),
        cluster: "".to_string(),
        starts_at: 0,
        segment: "".to_string(),
        audience: "".to_string(),
        external_ref: "".to_string(),
        status: LegacySegmentProposalStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_segment_proposal("code-249", "label-491");
        assert_eq!(item.status, LegacySegmentProposalStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_legacy_segment_proposal("code-249", "label-491");
        if item.move_to(LegacySegmentProposalStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacySegmentProposalStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_segment_proposal("code-249", "label-491");
        item.status = LegacySegmentProposalStatus::Archived;
        assert!(
            item.move_to(LegacySegmentProposalStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
