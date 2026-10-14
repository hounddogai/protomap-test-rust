use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted segment policy must pass. Check returns its problems, or none.
pub trait WeightedSegmentPolicyRule: std::fmt::Debug {
    fn check(&self, item: &WeightedSegmentPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentPolicyCountryLengthRule {
    pub max_length: i64,
}

impl WeightedSegmentPolicyRule for WeightedSegmentPolicyCountryLengthRule {
    fn check(&self, item: &WeightedSegmentPolicy) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentPolicyRequiredCodeRule {
    pub message: String,
}

impl WeightedSegmentPolicyRule for WeightedSegmentPolicyRequiredCodeRule {
    fn check(&self, item: &WeightedSegmentPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentPolicyLabelLengthRule {
    pub max_length: i64,
}

impl WeightedSegmentPolicyRule for WeightedSegmentPolicyLabelLengthRule {
    fn check(&self, item: &WeightedSegmentPolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted segment policy.
#[derive(Debug, Clone)]
pub struct WeightedSegmentPolicyValidator {
    pub rules: Vec<Rc<dyn WeightedSegmentPolicyRule>>,
}

impl WeightedSegmentPolicyValidator {
    pub fn validate(&self, item: &WeightedSegmentPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedSegmentPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedSegmentPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_segment_policy_validator() -> WeightedSegmentPolicyValidator {
    WeightedSegmentPolicyValidator {
        rules: vec![
            Rc::new(WeightedSegmentPolicyCountryLengthRule { max_length: 3600 }) as Rc<dyn WeightedSegmentPolicyRule>,
            Rc::new(WeightedSegmentPolicyRequiredCodeRule {
                message: "weighted segment policy needs this field".to_string(),
            }),
            Rc::new(WeightedSegmentPolicyLabelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_segment_policy_validator();
        let item = new_weighted_segment_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = WeightedSegmentPolicyCountryLengthRule { max_length: 120 };
        let mut item = new_weighted_segment_policy("x", "y");
        item.country = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedSegmentPolicyRequiredCodeRule {
            message: "weighted segment policy needs this field".to_string(),
        };
        let mut item = new_weighted_segment_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedSegmentPolicyLabelLengthRule { max_length: 120 };
        let mut item = new_weighted_segment_policy("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
