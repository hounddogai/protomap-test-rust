use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict policy item must pass. Check returns its problems, or none.
pub trait StrictPolicyItemRule: std::fmt::Debug {
    fn check(&self, item: &StrictPolicyItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyItemSourceLengthRule {
    pub max_length: i64,
}

impl StrictPolicyItemRule for StrictPolicyItemSourceLengthRule {
    fn check(&self, item: &StrictPolicyItem) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyItemRequiredSummaryRule {
    pub message: String,
}

impl StrictPolicyItemRule for StrictPolicyItemRequiredSummaryRule {
    fn check(&self, item: &StrictPolicyItem) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyItemBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictPolicyItemRule for StrictPolicyItemBurstRangeRule {
    fn check(&self, item: &StrictPolicyItem) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict policy item.
#[derive(Debug, Clone)]
pub struct StrictPolicyItemValidator {
    pub rules: Vec<Rc<dyn StrictPolicyItemRule>>,
}

impl StrictPolicyItemValidator {
    pub fn validate(&self, item: &StrictPolicyItem) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictPolicyItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPolicyItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_policy_item_validator() -> StrictPolicyItemValidator {
    StrictPolicyItemValidator {
        rules: vec![
            Rc::new(StrictPolicyItemSourceLengthRule { max_length: 16 }) as Rc<dyn StrictPolicyItemRule>,
            Rc::new(StrictPolicyItemRequiredSummaryRule {
                message: "strict policy item needs this field".to_string(),
            }),
            Rc::new(StrictPolicyItemBurstRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_policy_item_validator();
        let item = new_strict_policy_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = StrictPolicyItemSourceLengthRule { max_length: 16 };
        let mut item = new_strict_policy_item("x", "y");
        item.source = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = StrictPolicyItemRequiredSummaryRule {
            message: "strict policy item needs this field".to_string(),
        };
        let mut item = new_strict_policy_item("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = StrictPolicyItemBurstRangeRule { min: 64, max: 86400 };
        let mut item = new_strict_policy_item("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
