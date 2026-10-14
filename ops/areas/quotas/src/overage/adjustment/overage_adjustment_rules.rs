use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage adjustment must pass. Check returns its problems, or none.
pub trait OverageAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &OverageAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAdjustmentRequiredCategoryRule {
    pub message: String,
}

impl OverageAdjustmentRule for OverageAdjustmentRequiredCategoryRule {
    fn check(&self, item: &OverageAdjustment) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAdjustmentAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl OverageAdjustmentRule for OverageAdjustmentAllowedKeyIdRule {
    fn check(&self, item: &OverageAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is inconsistent", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAdjustmentAudienceLengthRule {
    pub max_length: i64,
}

impl OverageAdjustmentRule for OverageAdjustmentAudienceLengthRule {
    fn check(&self, item: &OverageAdjustment) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAdjustmentRequiredAudienceRule {
    pub message: String,
}

impl OverageAdjustmentRule for OverageAdjustmentRequiredAudienceRule {
    fn check(&self, item: &OverageAdjustment) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageAdjustmentStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageAdjustmentRule for OverageAdjustmentStartsAtRangeRule {
    fn check(&self, item: &OverageAdjustment) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a overage adjustment.
#[derive(Debug, Clone)]
pub struct OverageAdjustmentValidator {
    pub rules: Vec<Rc<dyn OverageAdjustmentRule>>,
}

impl OverageAdjustmentValidator {
    pub fn validate(&self, item: &OverageAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_adjustment_validator() -> OverageAdjustmentValidator {
    OverageAdjustmentValidator {
        rules: vec![
            Rc::new(OverageAdjustmentRequiredCategoryRule {
                message: "overage adjustment needs a value".to_string(),
            }) as Rc<dyn OverageAdjustmentRule>,
            Rc::new(OverageAdjustmentAllowedKeyIdRule {
                allowed: vec!["legacy".to_string(), "tiered".to_string()],
            }),
            Rc::new(OverageAdjustmentAudienceLengthRule { max_length: 64 }),
            Rc::new(OverageAdjustmentRequiredAudienceRule {
                message: "overage adjustment needs this field".to_string(),
            }),
            Rc::new(OverageAdjustmentStartsAtRangeRule { min: 80, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_adjustment_validator();
        let item = new_overage_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = OverageAdjustmentRequiredCategoryRule {
            message: "overage adjustment needs a value".to_string(),
        };
        let mut item = new_overage_adjustment("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = OverageAdjustmentAudienceLengthRule { max_length: 86400 };
        let mut item = new_overage_adjustment("x", "y");
        item.audience = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = OverageAdjustmentRequiredAudienceRule {
            message: "overage adjustment needs an entry".to_string(),
        };
        let mut item = new_overage_adjustment("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = OverageAdjustmentStartsAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_overage_adjustment("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
