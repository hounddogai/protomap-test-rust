use super::*;

use acme_kit::text;

/// Where a legacy binding proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyBindingProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyBindingProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyBindingProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy binding proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingProposal {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub attempts: i64,
    pub floor_value: f64,
    pub status: LegacyBindingProposalStatus,
}

impl LegacyBindingProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyBindingProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyBindingProposalStatus) -> bool {
        match self.status {
            LegacyBindingProposalStatus::Created => {
                match next {
                    LegacyBindingProposalStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyBindingProposalStatus::Validated => {
                return next == LegacyBindingProposalStatus::Published || next == LegacyBindingProposalStatus::Retired;
            }
            LegacyBindingProposalStatus::Published => {
                return next == LegacyBindingProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyBindingProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy binding proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.attempts as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyBindingProposal {
        LegacyBindingProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            attempts: self.attempts,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy binding proposal in status created.
pub fn new_legacy_binding_proposal(code: &str, title: &str) -> LegacyBindingProposal {
    LegacyBindingProposal {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        attempts: 0,
        floor_value: 0.0,
        status: LegacyBindingProposalStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_binding_proposal("code-399", "title-202");
        assert_eq!(item.status, LegacyBindingProposalStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_legacy_binding_proposal("code-399", "title-202");
        if item.move_to(LegacyBindingProposalStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyBindingProposalStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_binding_proposal("code-399", "title-202");
        item.status = LegacyBindingProposalStatus::Retired;
        assert!(
            item.move_to(LegacyBindingProposalStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
