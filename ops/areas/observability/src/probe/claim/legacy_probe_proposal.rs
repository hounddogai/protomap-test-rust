use super::*;

use acme_kit::text;

/// Where a legacy probe proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyProbeProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyProbeProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyProbeProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy probe proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeProposal {
    pub code: String,
    pub title: String,
    pub source: String,
    pub batch_size: i64,
    pub rate: f64,
    pub score: f64,
    pub status: LegacyProbeProposalStatus,
}

impl LegacyProbeProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyProbeProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyProbeProposalStatus) -> bool {
        match self.status {
            LegacyProbeProposalStatus::Created => {
                return next == LegacyProbeProposalStatus::Validated;
            }
            LegacyProbeProposalStatus::Validated => {
                return next == LegacyProbeProposalStatus::Published;
            }
            LegacyProbeProposalStatus::Published => {
                match next {
                    LegacyProbeProposalStatus::Retired => {
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

    pub fn move_to(&mut self, next: LegacyProbeProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy probe proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        (self.rate * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyProbeProposal {
        LegacyProbeProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            batch_size: self.batch_size,
            rate: self.rate,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy probe proposal in status created.
pub fn new_legacy_probe_proposal(code: &str, title: &str) -> LegacyProbeProposal {
    LegacyProbeProposal {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        batch_size: 0,
        rate: 0.0,
        score: 0.0,
        status: LegacyProbeProposalStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_probe_proposal("code-589", "title-777");
        assert_eq!(item.status, LegacyProbeProposalStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_legacy_probe_proposal("code-589", "title-777");
        if item.move_to(LegacyProbeProposalStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyProbeProposalStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_probe_proposal("code-589", "title-777");
        item.status = LegacyProbeProposalStatus::Retired;
        assert!(
            item.move_to(LegacyProbeProposalStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
