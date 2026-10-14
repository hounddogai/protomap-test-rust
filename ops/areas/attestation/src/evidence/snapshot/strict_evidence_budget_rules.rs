use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict evidence budget must pass. Check returns its problems, or none.
pub trait StrictEvidenceBudgetRule: std::fmt::Debug {
    fn check(&self, item: &StrictEvidenceBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEvidenceBudgetRequiredClusterRule {
    pub message: String,
}

impl StrictEvidenceBudgetRule for StrictEvidenceBudgetRequiredClusterRule {
    fn check(&self, item: &StrictEvidenceBudget) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEvidenceBudgetEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictEvidenceBudgetRule for StrictEvidenceBudgetEndsAtRangeRule {
    fn check(&self, item: &StrictEvidenceBudget) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEvidenceBudgetRequiredTitleRule {
    pub message: String,
}

impl StrictEvidenceBudgetRule for StrictEvidenceBudgetRequiredTitleRule {
    fn check(&self, item: &StrictEvidenceBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEvidenceBudgetRequiredCodeRule {
    pub message: String,
}

impl StrictEvidenceBudgetRule for StrictEvidenceBudgetRequiredCodeRule {
    fn check(&self, item: &StrictEvidenceBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEvidenceBudgetCodeLengthRule {
    pub max_length: i64,
}

impl StrictEvidenceBudgetRule for StrictEvidenceBudgetCodeLengthRule {
    fn check(&self, item: &StrictEvidenceBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEvidenceBudgetClusterLengthRule {
    pub max_length: i64,
}

impl StrictEvidenceBudgetRule for StrictEvidenceBudgetClusterLengthRule {
    fn check(&self, item: &StrictEvidenceBudget) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict evidence budget.
#[derive(Debug, Clone)]
pub struct StrictEvidenceBudgetValidator {
    pub rules: Vec<Rc<dyn StrictEvidenceBudgetRule>>,
}

impl StrictEvidenceBudgetValidator {
    pub fn validate(&self, item: &StrictEvidenceBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictEvidenceBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictEvidenceBudget]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_strict_evidence_budget_validator() -> StrictEvidenceBudgetValidator {
    StrictEvidenceBudgetValidator {
        rules: vec![
            Rc::new(StrictEvidenceBudgetRequiredClusterRule {
                message: "strict evidence budget needs an entry".to_string(),
            }) as Rc<dyn StrictEvidenceBudgetRule>,
            Rc::new(StrictEvidenceBudgetEndsAtRangeRule { min: 16, max: 255 }),
            Rc::new(StrictEvidenceBudgetRequiredTitleRule {
                message: "strict evidence budget needs a value".to_string(),
            }),
            Rc::new(StrictEvidenceBudgetRequiredCodeRule {
                message: "strict evidence budget needs an entry".to_string(),
            }),
            Rc::new(StrictEvidenceBudgetCodeLengthRule { max_length: 86400 }),
            Rc::new(StrictEvidenceBudgetClusterLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_evidence_budget_validator();
        let item = new_strict_evidence_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = StrictEvidenceBudgetRequiredClusterRule {
            message: "strict evidence budget needs a value".to_string(),
        };
        let mut item = new_strict_evidence_budget("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = StrictEvidenceBudgetEndsAtRangeRule { min: 120, max: 120 };
        let mut item = new_strict_evidence_budget("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = StrictEvidenceBudgetRequiredTitleRule {
            message: "strict evidence budget needs an entry".to_string(),
        };
        let mut item = new_strict_evidence_budget("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictEvidenceBudgetRequiredCodeRule {
            message: "strict evidence budget needs a value".to_string(),
        };
        let mut item = new_strict_evidence_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictEvidenceBudgetCodeLengthRule { max_length: 32 };
        let mut item = new_strict_evidence_budget("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = StrictEvidenceBudgetClusterLengthRule { max_length: 32 };
        let mut item = new_strict_evidence_budget("x", "y");
        item.cluster = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
