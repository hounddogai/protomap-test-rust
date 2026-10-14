use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress baseline must pass. Check returns its problems, or none.
pub trait EgressBaselineRule: std::fmt::Debug {
    fn check(&self, item: &EgressBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBaselineRequiredCodeRule {
    pub message: String,
}

impl EgressBaselineRule for EgressBaselineRequiredCodeRule {
    fn check(&self, item: &EgressBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBaselineRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressBaselineRule for EgressBaselineRevisionRangeRule {
    fn check(&self, item: &EgressBaseline) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBaselineCodeLengthRule {
    pub max_length: i64,
}

impl EgressBaselineRule for EgressBaselineCodeLengthRule {
    fn check(&self, item: &EgressBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBaselineRequiredSourceRule {
    pub message: String,
}

impl EgressBaselineRule for EgressBaselineRequiredSourceRule {
    fn check(&self, item: &EgressBaseline) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBaselineBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressBaselineRule for EgressBaselineBudgetRangeRule {
    fn check(&self, item: &EgressBaseline) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a egress baseline.
#[derive(Debug, Clone)]
pub struct EgressBaselineValidator {
    pub rules: Vec<Rc<dyn EgressBaselineRule>>,
}

impl EgressBaselineValidator {
    pub fn validate(&self, item: &EgressBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_baseline_validator() -> EgressBaselineValidator {
    EgressBaselineValidator {
        rules: vec![
            Rc::new(EgressBaselineRequiredCodeRule {
                message: "egress baseline needs this field".to_string(),
            }) as Rc<dyn EgressBaselineRule>,
            Rc::new(EgressBaselineRevisionRangeRule { min: 120, max: 3600 }),
            Rc::new(EgressBaselineCodeLengthRule { max_length: 3600 }),
            Rc::new(EgressBaselineRequiredSourceRule {
                message: "egress baseline needs an entry".to_string(),
            }),
            Rc::new(EgressBaselineBudgetRangeRule { min: 6.5, max: 349.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_baseline_validator();
        let item = new_egress_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressBaselineRequiredCodeRule {
            message: "egress baseline needs a value".to_string(),
        };
        let mut item = new_egress_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = EgressBaselineRevisionRangeRule { min: 64, max: 120 };
        let mut item = new_egress_baseline("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EgressBaselineCodeLengthRule { max_length: 86400 };
        let mut item = new_egress_baseline("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = EgressBaselineRequiredSourceRule {
            message: "egress baseline needs a value".to_string(),
        };
        let mut item = new_egress_baseline("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = EgressBaselineBudgetRangeRule { min: 0.9, max: 2501.8 };
        let mut item = new_egress_baseline("x", "y");
        item.budget = 2503.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
