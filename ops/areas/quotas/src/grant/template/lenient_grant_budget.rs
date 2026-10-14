use super::*;

use acme_kit::text;

/// Where a lenient grant budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientGrantBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientGrantBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientGrantBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient grant budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantBudget {
    pub code: String,
    pub title: String,
    pub category: String,
    pub revision: i64,
    pub source: String,
    pub maximum: f64,
    pub status: LenientGrantBudgetStatus,
}

impl LenientGrantBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientGrantBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientGrantBudgetStatus) -> bool {
        match self.status {
            LenientGrantBudgetStatus::Created => {
                return next == LenientGrantBudgetStatus::Published;
            }
            LenientGrantBudgetStatus::Validated => {
                return next == LenientGrantBudgetStatus::Retired;
            }
            LenientGrantBudgetStatus::Published => {
                return next == LenientGrantBudgetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientGrantBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient grant budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.revision as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientGrantBudget {
        LenientGrantBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            revision: self.revision,
            source: self.source.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a lenient grant budget in status created.
pub fn new_lenient_grant_budget(code: &str, title: &str) -> LenientGrantBudget {
    LenientGrantBudget {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        revision: 0,
        source: "".to_string(),
        maximum: 0.0,
        status: LenientGrantBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_grant_budget("code-16", "title-705");
        assert_eq!(item.status, LenientGrantBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_lenient_grant_budget("code-16", "title-705");
        if item.move_to(LenientGrantBudgetStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientGrantBudgetStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_grant_budget("code-16", "title-705");
        item.status = LenientGrantBudgetStatus::Retired;
        assert!(
            item.move_to(LenientGrantBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
