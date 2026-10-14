use super::*;

use acme_kit::text;

/// Where a seasonal region pin proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRegionPinProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalRegionPinProposalStatus {
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

impl std::fmt::Display for SeasonalRegionPinProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal region pin proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRegionPinProposal {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub rate: f64,
    pub external_ref: String,
    pub source: String,
    pub region: String,
    pub fingerprint: String,
    pub sequence: i64,
    pub revision: i64,
    pub status: SeasonalRegionPinProposalStatus,
}

impl SeasonalRegionPinProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRegionPinProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRegionPinProposalStatus) -> bool {
        match self.status {
            SeasonalRegionPinProposalStatus::New => {
                match next {
                    SeasonalRegionPinProposalStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRegionPinProposalStatus::Queued => {
                return next == SeasonalRegionPinProposalStatus::Done;
            }
            SeasonalRegionPinProposalStatus::Running => {
                return next == SeasonalRegionPinProposalStatus::Failed;
            }
            SeasonalRegionPinProposalStatus::Done => {
                match next {
                    SeasonalRegionPinProposalStatus::Failed => {
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

    pub fn move_to(&mut self, next: SeasonalRegionPinProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal region pin proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.revision as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRegionPinProposal {
        SeasonalRegionPinProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            rate: self.rate,
            external_ref: self.external_ref.clone(),
            source: self.source.clone(),
            region: self.region.clone(),
            fingerprint: self.fingerprint.clone(),
            sequence: self.sequence,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a seasonal region pin proposal in status new.
pub fn new_seasonal_region_pin_proposal(code: &str, label: &str) -> SeasonalRegionPinProposal {
    SeasonalRegionPinProposal {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        rate: 0.0,
        external_ref: "".to_string(),
        source: "".to_string(),
        region: "".to_string(),
        fingerprint: "".to_string(),
        sequence: 0,
        revision: 0,
        status: SeasonalRegionPinProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_region_pin_proposal("code-161", "label-702");
        assert_eq!(item.status, SeasonalRegionPinProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_seasonal_region_pin_proposal("code-161", "label-702");
        if item.move_to(SeasonalRegionPinProposalStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalRegionPinProposalStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_region_pin_proposal("code-161", "label-702");
        item.status = SeasonalRegionPinProposalStatus::Failed;
        assert!(
            item.move_to(SeasonalRegionPinProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
