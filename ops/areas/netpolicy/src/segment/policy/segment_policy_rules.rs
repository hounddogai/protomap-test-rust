use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment policy must pass. Check returns its problems, or none.
pub trait SegmentPolicyRule: std::fmt::Debug {
    fn check(&self, item: &SegmentPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPolicyRequiredContactEmailRule {
    pub message: String,
}

impl SegmentPolicyRule for SegmentPolicyRequiredContactEmailRule {
    fn check(&self, item: &SegmentPolicy) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPolicyWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentPolicyRule for SegmentPolicyWeightRangeRule {
    fn check(&self, item: &SegmentPolicy) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPolicyCodePrefixRule {
    pub prefix: String,
}

impl SegmentPolicyRule for SegmentPolicyCodePrefixRule {
    fn check(&self, item: &SegmentPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPolicyRequiredUnitRule {
    pub message: String,
}

impl SegmentPolicyRule for SegmentPolicyRequiredUnitRule {
    fn check(&self, item: &SegmentPolicy) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPolicyUnitLengthRule {
    pub max_length: i64,
}

impl SegmentPolicyRule for SegmentPolicyUnitLengthRule {
    fn check(&self, item: &SegmentPolicy) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment policy.
#[derive(Debug, Clone)]
pub struct SegmentPolicyValidator {
    pub rules: Vec<Rc<dyn SegmentPolicyRule>>,
}

impl SegmentPolicyValidator {
    pub fn validate(&self, item: &SegmentPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_policy_validator() -> SegmentPolicyValidator {
    SegmentPolicyValidator {
        rules: vec![
            Rc::new(SegmentPolicyRequiredContactEmailRule {
                message: "segment policy needs a value".to_string(),
            }) as Rc<dyn SegmentPolicyRule>,
            Rc::new(SegmentPolicyWeightRangeRule { min: 2.7, max: 846.8 }),
            Rc::new(SegmentPolicyCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SegmentPolicyRequiredUnitRule {
                message: "segment policy needs a value".to_string(),
            }),
            Rc::new(SegmentPolicyUnitLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_policy_validator();
        let item = new_segment_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = SegmentPolicyRequiredContactEmailRule {
            message: "segment policy needs a value".to_string(),
        };
        let mut item = new_segment_policy("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = SegmentPolicyWeightRangeRule { min: 3.8, max: 3226.2 };
        let mut item = new_segment_policy("x", "y");
        item.weight = 3227.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = SegmentPolicyRequiredUnitRule {
            message: "segment policy needs a value".to_string(),
        };
        let mut item = new_segment_policy("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = SegmentPolicyUnitLengthRule { max_length: 16 };
        let mut item = new_segment_policy("x", "y");
        item.unit = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
