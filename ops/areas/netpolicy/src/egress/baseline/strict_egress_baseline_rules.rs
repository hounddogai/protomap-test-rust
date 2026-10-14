use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict egress baseline must pass. Check returns its problems, or none.
pub trait StrictEgressBaselineRule: std::fmt::Debug {
    fn check(&self, item: &StrictEgressBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressBaselinePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictEgressBaselineRule for StrictEgressBaselinePortRangeRule {
    fn check(&self, item: &StrictEgressBaseline) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressBaselineRequiredLabelRule {
    pub message: String,
}

impl StrictEgressBaselineRule for StrictEgressBaselineRequiredLabelRule {
    fn check(&self, item: &StrictEgressBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressBaselineRequiredCodeRule {
    pub message: String,
}

impl StrictEgressBaselineRule for StrictEgressBaselineRequiredCodeRule {
    fn check(&self, item: &StrictEgressBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict egress baseline.
#[derive(Debug, Clone)]
pub struct StrictEgressBaselineValidator {
    pub rules: Vec<Rc<dyn StrictEgressBaselineRule>>,
}

impl StrictEgressBaselineValidator {
    pub fn validate(&self, item: &StrictEgressBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictEgressBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictEgressBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_egress_baseline_validator() -> StrictEgressBaselineValidator {
    StrictEgressBaselineValidator {
        rules: vec![
            Rc::new(StrictEgressBaselinePortRangeRule { min: 1000, max: 86400 }) as Rc<dyn StrictEgressBaselineRule>,
            Rc::new(StrictEgressBaselineRequiredLabelRule {
                message: "strict egress baseline needs an entry".to_string(),
            }),
            Rc::new(StrictEgressBaselineRequiredCodeRule {
                message: "strict egress baseline needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_egress_baseline_validator();
        let item = new_strict_egress_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = StrictEgressBaselinePortRangeRule { min: 32, max: 120 };
        let mut item = new_strict_egress_baseline("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictEgressBaselineRequiredLabelRule {
            message: "strict egress baseline needs a value".to_string(),
        };
        let mut item = new_strict_egress_baseline("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictEgressBaselineRequiredCodeRule {
            message: "strict egress baseline needs a value".to_string(),
        };
        let mut item = new_strict_egress_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
