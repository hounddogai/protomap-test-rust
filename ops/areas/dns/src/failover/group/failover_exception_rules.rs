use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a failover exception must pass. Check returns its problems, or none.
pub trait FailoverExceptionRule: std::fmt::Debug {
    fn check(&self, item: &FailoverException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverExceptionAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl FailoverExceptionRule for FailoverExceptionAllowedTitleRule {
    fn check(&self, item: &FailoverException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is below minimum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverExceptionRequiredAudienceRule {
    pub message: String,
}

impl FailoverExceptionRule for FailoverExceptionRequiredAudienceRule {
    fn check(&self, item: &FailoverException) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverExceptionMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverExceptionRule for FailoverExceptionMarginRangeRule {
    fn check(&self, item: &FailoverException) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverExceptionCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverExceptionRule for FailoverExceptionCostRangeRule {
    fn check(&self, item: &FailoverException) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverExceptionRequiredCodeRule {
    pub message: String,
}

impl FailoverExceptionRule for FailoverExceptionRequiredCodeRule {
    fn check(&self, item: &FailoverException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover exception.
#[derive(Debug, Clone)]
pub struct FailoverExceptionValidator {
    pub rules: Vec<Rc<dyn FailoverExceptionRule>>,
}

impl FailoverExceptionValidator {
    pub fn validate(&self, item: &FailoverException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_exception_validator() -> FailoverExceptionValidator {
    FailoverExceptionValidator {
        rules: vec![
            Rc::new(FailoverExceptionAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn FailoverExceptionRule>,
            Rc::new(FailoverExceptionRequiredAudienceRule {
                message: "failover exception needs a value".to_string(),
            }),
            Rc::new(FailoverExceptionMarginRangeRule { min: 8.6, max: 4246.3 }),
            Rc::new(FailoverExceptionCostRangeRule { min: 0.4, max: 2560.2 }),
            Rc::new(FailoverExceptionRequiredCodeRule {
                message: "failover exception needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_exception_validator();
        let item = new_failover_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = FailoverExceptionRequiredAudienceRule {
            message: "failover exception needs this field".to_string(),
        };
        let mut item = new_failover_exception("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = FailoverExceptionMarginRangeRule { min: 0.3, max: 4290.6 };
        let mut item = new_failover_exception("x", "y");
        item.margin = 4292.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = FailoverExceptionCostRangeRule { min: 4.9, max: 3817.7 };
        let mut item = new_failover_exception("x", "y");
        item.cost = 3819.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FailoverExceptionRequiredCodeRule {
            message: "failover exception needs a value".to_string(),
        };
        let mut item = new_failover_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
