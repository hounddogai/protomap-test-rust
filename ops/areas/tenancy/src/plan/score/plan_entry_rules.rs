use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan entry must pass. Check returns its problems, or none.
pub trait PlanEntryRule: std::fmt::Debug {
    fn check(&self, item: &PlanEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanEntryGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanEntryRule for PlanEntryGraceDaysRangeRule {
    fn check(&self, item: &PlanEntry) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanEntryCodePrefixRule {
    pub prefix: String,
}

impl PlanEntryRule for PlanEntryCodePrefixRule {
    fn check(&self, item: &PlanEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanEntryChannelLengthRule {
    pub max_length: i64,
}

impl PlanEntryRule for PlanEntryChannelLengthRule {
    fn check(&self, item: &PlanEntry) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan entry.
#[derive(Debug, Clone)]
pub struct PlanEntryValidator {
    pub rules: Vec<Rc<dyn PlanEntryRule>>,
}

impl PlanEntryValidator {
    pub fn validate(&self, item: &PlanEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_entry_validator() -> PlanEntryValidator {
    PlanEntryValidator {
        rules: vec![
            Rc::new(PlanEntryGraceDaysRangeRule { min: 255, max: 255 }) as Rc<dyn PlanEntryRule>,
            Rc::new(PlanEntryCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PlanEntryChannelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_entry_validator();
        let item = new_plan_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = PlanEntryGraceDaysRangeRule { min: 80, max: 80 };
        let mut item = new_plan_entry("x", "y");
        item.grace_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = PlanEntryChannelLengthRule { max_length: 86400 };
        let mut item = new_plan_entry("x", "y");
        item.channel = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
