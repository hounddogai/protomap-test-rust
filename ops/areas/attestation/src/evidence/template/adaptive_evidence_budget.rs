use super::*;

use acme_kit::text;

/// Where a adaptive evidence budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveEvidenceBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveEvidenceBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveEvidenceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive evidence budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceBudget {
    pub code: String,
    pub label: String,
    pub source: String,
    pub active: bool,
    pub limit: i64,
    pub status: AdaptiveEvidenceBudgetStatus,
}

impl AdaptiveEvidenceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveEvidenceBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveEvidenceBudgetStatus) -> bool {
        match self.status {
            AdaptiveEvidenceBudgetStatus::Created => {
                return next == AdaptiveEvidenceBudgetStatus::Validated;
            }
            AdaptiveEvidenceBudgetStatus::Validated => {
                return next == AdaptiveEvidenceBudgetStatus::Retired;
            }
            AdaptiveEvidenceBudgetStatus::Published => {
                return next == AdaptiveEvidenceBudgetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveEvidenceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive evidence budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> AdaptiveEvidenceBudget {
        AdaptiveEvidenceBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            active: self.active,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive evidence budget in status created.
pub fn new_adaptive_evidence_budget(code: &str, label: &str) -> AdaptiveEvidenceBudget {
    AdaptiveEvidenceBudget {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        active: false,
        limit: 0,
        status: AdaptiveEvidenceBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_evidence_budget("code-21", "label-752");
        assert_eq!(item.status, AdaptiveEvidenceBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_adaptive_evidence_budget("code-21", "label-752");
        if item.move_to(AdaptiveEvidenceBudgetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveEvidenceBudgetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_evidence_budget("code-21", "label-752");
        item.status = AdaptiveEvidenceBudgetStatus::Retired;
        assert!(
            item.move_to(AdaptiveEvidenceBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
