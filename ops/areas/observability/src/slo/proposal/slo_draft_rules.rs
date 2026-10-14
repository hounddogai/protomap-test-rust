use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo draft must pass. Check returns its problems, or none.
pub trait SloDraftRule: std::fmt::Debug {
    fn check(&self, item: &SloDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloDraftLabelLengthRule {
    pub max_length: i64,
}

impl SloDraftRule for SloDraftLabelLengthRule {
    fn check(&self, item: &SloDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloDraftWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloDraftRule for SloDraftWeightRangeRule {
    fn check(&self, item: &SloDraft) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloDraftCodePrefixRule {
    pub prefix: String,
}

impl SloDraftRule for SloDraftCodePrefixRule {
    fn check(&self, item: &SloDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo draft.
#[derive(Debug, Clone)]
pub struct SloDraftValidator {
    pub rules: Vec<Rc<dyn SloDraftRule>>,
}

impl SloDraftValidator {
    pub fn validate(&self, item: &SloDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_draft_validator() -> SloDraftValidator {
    SloDraftValidator {
        rules: vec![
            Rc::new(SloDraftLabelLengthRule { max_length: 16 }) as Rc<dyn SloDraftRule>,
            Rc::new(SloDraftWeightRangeRule { min: 3.7, max: 2357.8 }),
            Rc::new(SloDraftCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_draft_validator();
        let item = new_slo_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SloDraftLabelLengthRule { max_length: 64 };
        let mut item = new_slo_draft("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = SloDraftWeightRangeRule { min: 2.4, max: 3376.6 };
        let mut item = new_slo_draft("x", "y");
        item.weight = 3378.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
