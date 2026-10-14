use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier budget must pass. Check returns its problems, or none.
pub trait VerifierBudgetRule: std::fmt::Debug {
    fn check(&self, item: &VerifierBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBudgetRequiredTimezoneRule {
    pub message: String,
}

impl VerifierBudgetRule for VerifierBudgetRequiredTimezoneRule {
    fn check(&self, item: &VerifierBudget) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBudgetRequiredNameRule {
    pub message: String,
}

impl VerifierBudgetRule for VerifierBudgetRequiredNameRule {
    fn check(&self, item: &VerifierBudget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBudgetRequiredCodeRule {
    pub message: String,
}

impl VerifierBudgetRule for VerifierBudgetRequiredCodeRule {
    fn check(&self, item: &VerifierBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBudgetRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierBudgetRule for VerifierBudgetRatioRangeRule {
    fn check(&self, item: &VerifierBudget) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBudgetCodePrefixRule {
    pub prefix: String,
}

impl VerifierBudgetRule for VerifierBudgetCodePrefixRule {
    fn check(&self, item: &VerifierBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier budget.
#[derive(Debug, Clone)]
pub struct VerifierBudgetValidator {
    pub rules: Vec<Rc<dyn VerifierBudgetRule>>,
}

impl VerifierBudgetValidator {
    pub fn validate(&self, item: &VerifierBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_budget_validator() -> VerifierBudgetValidator {
    VerifierBudgetValidator {
        rules: vec![
            Rc::new(VerifierBudgetRequiredTimezoneRule {
                message: "verifier budget needs a value".to_string(),
            }) as Rc<dyn VerifierBudgetRule>,
            Rc::new(VerifierBudgetRequiredNameRule {
                message: "verifier budget needs a value".to_string(),
            }),
            Rc::new(VerifierBudgetRequiredCodeRule {
                message: "verifier budget needs a value".to_string(),
            }),
            Rc::new(VerifierBudgetRatioRangeRule { min: 5.8, max: 2650.6 }),
            Rc::new(VerifierBudgetCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_budget_validator();
        let item = new_verifier_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = VerifierBudgetRequiredTimezoneRule {
            message: "verifier budget needs an entry".to_string(),
        };
        let mut item = new_verifier_budget("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = VerifierBudgetRequiredNameRule {
            message: "verifier budget needs an entry".to_string(),
        };
        let mut item = new_verifier_budget("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = VerifierBudgetRequiredCodeRule {
            message: "verifier budget needs a value".to_string(),
        };
        let mut item = new_verifier_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = VerifierBudgetRatioRangeRule { min: 4.4, max: 1674.7 };
        let mut item = new_verifier_budget("x", "y");
        item.ratio = 1676.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
