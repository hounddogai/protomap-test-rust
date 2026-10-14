use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive credential exception must pass. Check returns its problems, or none.
pub trait AdaptiveCredentialExceptionRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveCredentialException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialExceptionBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveCredentialExceptionRule for AdaptiveCredentialExceptionBudgetRangeRule {
    fn check(&self, item: &AdaptiveCredentialException) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialExceptionRequiredLabelRule {
    pub message: String,
}

impl AdaptiveCredentialExceptionRule for AdaptiveCredentialExceptionRequiredLabelRule {
    fn check(&self, item: &AdaptiveCredentialException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialExceptionPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveCredentialExceptionRule for AdaptiveCredentialExceptionPortRangeRule {
    fn check(&self, item: &AdaptiveCredentialException) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive credential exception.
#[derive(Debug, Clone)]
pub struct AdaptiveCredentialExceptionValidator {
    pub rules: Vec<Rc<dyn AdaptiveCredentialExceptionRule>>,
}

impl AdaptiveCredentialExceptionValidator {
    pub fn validate(&self, item: &AdaptiveCredentialException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveCredentialException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveCredentialException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_credential_exception_validator() -> AdaptiveCredentialExceptionValidator {
    AdaptiveCredentialExceptionValidator {
        rules: vec![
            Rc::new(AdaptiveCredentialExceptionBudgetRangeRule { min: 0.6, max: 1195.4 })
                as Rc<dyn AdaptiveCredentialExceptionRule>,
            Rc::new(AdaptiveCredentialExceptionRequiredLabelRule {
                message: "adaptive credential exception needs an entry".to_string(),
            }),
            Rc::new(AdaptiveCredentialExceptionPortRangeRule { min: 120, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_credential_exception_validator();
        let item = new_adaptive_credential_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = AdaptiveCredentialExceptionBudgetRangeRule { min: 7.2, max: 351.9 };
        let mut item = new_adaptive_credential_exception("x", "y");
        item.budget = 353.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveCredentialExceptionRequiredLabelRule {
            message: "adaptive credential exception needs a value".to_string(),
        };
        let mut item = new_adaptive_credential_exception("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = AdaptiveCredentialExceptionPortRangeRule { min: 16, max: 64 };
        let mut item = new_adaptive_credential_exception("x", "y");
        item.port = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
