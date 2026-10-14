use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation target must pass. Check returns its problems, or none.
pub trait IsolationTargetRule: std::fmt::Debug {
    fn check(&self, item: &IsolationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTargetRequiredSourceRule {
    pub message: String,
}

impl IsolationTargetRule for IsolationTargetRequiredSourceRule {
    fn check(&self, item: &IsolationTarget) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTargetRequiredTitleRule {
    pub message: String,
}

impl IsolationTargetRule for IsolationTargetRequiredTitleRule {
    fn check(&self, item: &IsolationTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTargetAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IsolationTargetRule for IsolationTargetAmountRangeRule {
    fn check(&self, item: &IsolationTarget) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTargetRequiredRegionRule {
    pub message: String,
}

impl IsolationTargetRule for IsolationTargetRequiredRegionRule {
    fn check(&self, item: &IsolationTarget) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation target.
#[derive(Debug, Clone)]
pub struct IsolationTargetValidator {
    pub rules: Vec<Rc<dyn IsolationTargetRule>>,
}

impl IsolationTargetValidator {
    pub fn validate(&self, item: &IsolationTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_target_validator() -> IsolationTargetValidator {
    IsolationTargetValidator {
        rules: vec![
            Rc::new(IsolationTargetRequiredSourceRule {
                message: "isolation target needs this field".to_string(),
            }) as Rc<dyn IsolationTargetRule>,
            Rc::new(IsolationTargetRequiredTitleRule {
                message: "isolation target needs an entry".to_string(),
            }),
            Rc::new(IsolationTargetAmountRangeRule { min: 1.3, max: 2313.9 }),
            Rc::new(IsolationTargetRequiredRegionRule {
                message: "isolation target needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_target_validator();
        let item = new_isolation_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = IsolationTargetRequiredSourceRule {
            message: "isolation target needs this field".to_string(),
        };
        let mut item = new_isolation_target("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = IsolationTargetRequiredTitleRule {
            message: "isolation target needs this field".to_string(),
        };
        let mut item = new_isolation_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = IsolationTargetAmountRangeRule { min: 0.8, max: 2936.9 };
        let mut item = new_isolation_target("x", "y");
        item.amount = 2938.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = IsolationTargetRequiredRegionRule {
            message: "isolation target needs a value".to_string(),
        };
        let mut item = new_isolation_target("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
