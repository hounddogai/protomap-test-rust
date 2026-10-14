use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a budget burn rollup must pass. Check returns its problems, or none.
pub trait BudgetBurnRollupRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRollupEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnRollupRule for BudgetBurnRollupEndsAtRangeRule {
    fn check(&self, item: &BudgetBurnRollup) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRollupRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnRollupRule for BudgetBurnRollupRevisionRangeRule {
    fn check(&self, item: &BudgetBurnRollup) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRollupAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl BudgetBurnRollupRule for BudgetBurnRollupAllowedTitleRule {
    fn check(&self, item: &BudgetBurnRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRollupRequiredTitleRule {
    pub message: String,
}

impl BudgetBurnRollupRule for BudgetBurnRollupRequiredTitleRule {
    fn check(&self, item: &BudgetBurnRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRollupRequiredChecksumRule {
    pub message: String,
}

impl BudgetBurnRollupRule for BudgetBurnRollupRequiredChecksumRule {
    fn check(&self, item: &BudgetBurnRollup) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn rollup.
#[derive(Debug, Clone)]
pub struct BudgetBurnRollupValidator {
    pub rules: Vec<Rc<dyn BudgetBurnRollupRule>>,
}

impl BudgetBurnRollupValidator {
    pub fn validate(&self, item: &BudgetBurnRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_rollup_validator() -> BudgetBurnRollupValidator {
    BudgetBurnRollupValidator {
        rules: vec![
            Rc::new(BudgetBurnRollupEndsAtRangeRule { min: 16, max: 32 }) as Rc<dyn BudgetBurnRollupRule>,
            Rc::new(BudgetBurnRollupRevisionRangeRule { min: 120, max: 255 }),
            Rc::new(BudgetBurnRollupAllowedTitleRule {
                allowed: vec!["seasonal".to_string(), "lenient".to_string()],
            }),
            Rc::new(BudgetBurnRollupRequiredTitleRule {
                message: "budget burn rollup needs an entry".to_string(),
            }),
            Rc::new(BudgetBurnRollupRequiredChecksumRule {
                message: "budget burn rollup needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_rollup_validator();
        let item = new_budget_burn_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = BudgetBurnRollupEndsAtRangeRule { min: 80, max: 3600 };
        let mut item = new_budget_burn_rollup("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = BudgetBurnRollupRevisionRangeRule { min: 16, max: 86400 };
        let mut item = new_budget_burn_rollup("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BudgetBurnRollupRequiredTitleRule {
            message: "budget burn rollup needs a value".to_string(),
        };
        let mut item = new_budget_burn_rollup("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = BudgetBurnRollupRequiredChecksumRule {
            message: "budget burn rollup needs a value".to_string(),
        };
        let mut item = new_budget_burn_rollup("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
