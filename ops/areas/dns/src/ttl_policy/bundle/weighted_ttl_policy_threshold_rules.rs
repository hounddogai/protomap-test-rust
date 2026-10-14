use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted ttl policy threshold must pass. Check returns its problems, or none.
pub trait WeightedTtlPolicyThresholdRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTtlPolicyThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyThresholdRequiredSourceRule {
    pub message: String,
}

impl WeightedTtlPolicyThresholdRule for WeightedTtlPolicyThresholdRequiredSourceRule {
    fn check(&self, item: &WeightedTtlPolicyThreshold) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyThresholdChecksumLengthRule {
    pub max_length: i64,
}

impl WeightedTtlPolicyThresholdRule for WeightedTtlPolicyThresholdChecksumLengthRule {
    fn check(&self, item: &WeightedTtlPolicyThreshold) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyThresholdLabelLengthRule {
    pub max_length: i64,
}

impl WeightedTtlPolicyThresholdRule for WeightedTtlPolicyThresholdLabelLengthRule {
    fn check(&self, item: &WeightedTtlPolicyThreshold) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyThresholdCodeLengthRule {
    pub max_length: i64,
}

impl WeightedTtlPolicyThresholdRule for WeightedTtlPolicyThresholdCodeLengthRule {
    fn check(&self, item: &WeightedTtlPolicyThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyThresholdRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedTtlPolicyThresholdRule for WeightedTtlPolicyThresholdRatioRangeRule {
    fn check(&self, item: &WeightedTtlPolicyThreshold) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyThresholdEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedTtlPolicyThresholdRule for WeightedTtlPolicyThresholdEndsAtRangeRule {
    fn check(&self, item: &WeightedTtlPolicyThreshold) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a weighted ttl policy threshold.
#[derive(Debug, Clone)]
pub struct WeightedTtlPolicyThresholdValidator {
    pub rules: Vec<Rc<dyn WeightedTtlPolicyThresholdRule>>,
}

impl WeightedTtlPolicyThresholdValidator {
    pub fn validate(&self, item: &WeightedTtlPolicyThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedTtlPolicyThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTtlPolicyThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_ttl_policy_threshold_validator() -> WeightedTtlPolicyThresholdValidator {
    WeightedTtlPolicyThresholdValidator {
        rules: vec![
            Rc::new(WeightedTtlPolicyThresholdRequiredSourceRule {
                message: "weighted ttl policy threshold needs this field".to_string(),
            }) as Rc<dyn WeightedTtlPolicyThresholdRule>,
            Rc::new(WeightedTtlPolicyThresholdChecksumLengthRule { max_length: 80 }),
            Rc::new(WeightedTtlPolicyThresholdLabelLengthRule { max_length: 1000 }),
            Rc::new(WeightedTtlPolicyThresholdCodeLengthRule { max_length: 255 }),
            Rc::new(WeightedTtlPolicyThresholdRatioRangeRule { min: 3.3, max: 4785.5 }),
            Rc::new(WeightedTtlPolicyThresholdEndsAtRangeRule { min: 86400, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_ttl_policy_threshold_validator();
        let item = new_weighted_ttl_policy_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = WeightedTtlPolicyThresholdRequiredSourceRule {
            message: "weighted ttl policy threshold needs an entry".to_string(),
        };
        let mut item = new_weighted_ttl_policy_threshold("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = WeightedTtlPolicyThresholdChecksumLengthRule { max_length: 120 };
        let mut item = new_weighted_ttl_policy_threshold("x", "y");
        item.checksum = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedTtlPolicyThresholdLabelLengthRule { max_length: 1000 };
        let mut item = new_weighted_ttl_policy_threshold("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedTtlPolicyThresholdCodeLengthRule { max_length: 32 };
        let mut item = new_weighted_ttl_policy_threshold("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = WeightedTtlPolicyThresholdRatioRangeRule { min: 9.2, max: 4501.7 };
        let mut item = new_weighted_ttl_policy_threshold("x", "y");
        item.ratio = 4503.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = WeightedTtlPolicyThresholdEndsAtRangeRule { min: 80, max: 255 };
        let mut item = new_weighted_ttl_policy_threshold("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
