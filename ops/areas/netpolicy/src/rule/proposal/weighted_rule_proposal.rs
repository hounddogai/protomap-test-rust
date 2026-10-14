use super::*;

use acme_kit::text;

/// Where a weighted rule proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRuleProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedRuleProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for WeightedRuleProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted rule proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleProposal {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub threshold: f64,
    pub version: i64,
    pub unit: String,
    pub category: String,
    pub phone: String,
    pub status: WeightedRuleProposalStatus,
}

impl WeightedRuleProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRuleProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRuleProposalStatus) -> bool {
        match self.status {
            WeightedRuleProposalStatus::Draft => {
                match next {
                    WeightedRuleProposalStatus::Pending | WeightedRuleProposalStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRuleProposalStatus::Pending => {
                return next == WeightedRuleProposalStatus::Active || next == WeightedRuleProposalStatus::Suspended;
            }
            WeightedRuleProposalStatus::Active => {
                return next == WeightedRuleProposalStatus::Suspended || next == WeightedRuleProposalStatus::Closed;
            }
            WeightedRuleProposalStatus::Suspended => {
                return next == WeightedRuleProposalStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRuleProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted rule proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.version as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRuleProposal {
        WeightedRuleProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            threshold: self.threshold,
            version: self.version,
            unit: self.unit.clone(),
            category: self.category.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a weighted rule proposal in status draft.
pub fn new_weighted_rule_proposal(code: &str, title: &str) -> WeightedRuleProposal {
    WeightedRuleProposal {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        threshold: 0.0,
        version: 0,
        unit: "".to_string(),
        category: "".to_string(),
        phone: "".to_string(),
        status: WeightedRuleProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_weighted_rule_proposal("code-707", "title-510");
        assert_eq!(item.status, WeightedRuleProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_weighted_rule_proposal("code-707", "title-510");
        if item.move_to(WeightedRuleProposalStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRuleProposalStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_weighted_rule_proposal("code-707", "title-510");
        item.status = WeightedRuleProposalStatus::Closed;
        assert!(
            item.move_to(WeightedRuleProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
