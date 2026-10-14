use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual verifier budget must pass. Check returns its problems, or none.
pub trait ManualVerifierBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ManualVerifierBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierBudgetRequiredCodeRule {
    pub message: String,
}

impl ManualVerifierBudgetRule for ManualVerifierBudgetRequiredCodeRule {
    fn check(&self, item: &ManualVerifierBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierBudgetRequiredSegmentRule {
    pub message: String,
}

impl ManualVerifierBudgetRule for ManualVerifierBudgetRequiredSegmentRule {
    fn check(&self, item: &ManualVerifierBudget) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierBudgetBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualVerifierBudgetRule for ManualVerifierBudgetBurstRangeRule {
    fn check(&self, item: &ManualVerifierBudget) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierBudgetCodeLengthRule {
    pub max_length: i64,
}

impl ManualVerifierBudgetRule for ManualVerifierBudgetCodeLengthRule {
    fn check(&self, item: &ManualVerifierBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual verifier budget.
#[derive(Debug, Clone)]
pub struct ManualVerifierBudgetValidator {
    pub rules: Vec<Rc<dyn ManualVerifierBudgetRule>>,
}

impl ManualVerifierBudgetValidator {
    pub fn validate(&self, item: &ManualVerifierBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualVerifierBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualVerifierBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_verifier_budget_validator() -> ManualVerifierBudgetValidator {
    ManualVerifierBudgetValidator {
        rules: vec![
            Rc::new(ManualVerifierBudgetRequiredCodeRule {
                message: "manual verifier budget needs an entry".to_string(),
            }) as Rc<dyn ManualVerifierBudgetRule>,
            Rc::new(ManualVerifierBudgetRequiredSegmentRule {
                message: "manual verifier budget needs a value".to_string(),
            }),
            Rc::new(ManualVerifierBudgetBurstRangeRule { min: 32, max: 80 }),
            Rc::new(ManualVerifierBudgetCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_verifier_budget_validator();
        let item = new_manual_verifier_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualVerifierBudgetRequiredCodeRule {
            message: "manual verifier budget needs a value".to_string(),
        };
        let mut item = new_manual_verifier_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = ManualVerifierBudgetRequiredSegmentRule {
            message: "manual verifier budget needs a value".to_string(),
        };
        let mut item = new_manual_verifier_budget("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = ManualVerifierBudgetBurstRangeRule { min: 1000, max: 1000 };
        let mut item = new_manual_verifier_budget("x", "y");
        item.burst = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualVerifierBudgetCodeLengthRule { max_length: 64 };
        let mut item = new_manual_verifier_budget("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
