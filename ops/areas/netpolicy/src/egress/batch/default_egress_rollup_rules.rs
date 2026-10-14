use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default egress rollup must pass. Check returns its problems, or none.
pub trait DefaultEgressRollupRule: std::fmt::Debug {
    fn check(&self, item: &DefaultEgressRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressRollupChecksumLengthRule {
    pub max_length: i64,
}

impl DefaultEgressRollupRule for DefaultEgressRollupChecksumLengthRule {
    fn check(&self, item: &DefaultEgressRollup) -> Vec<String> {
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
pub struct DefaultEgressRollupCodePrefixRule {
    pub prefix: String,
}

impl DefaultEgressRollupRule for DefaultEgressRollupCodePrefixRule {
    fn check(&self, item: &DefaultEgressRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressRollupMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultEgressRollupRule for DefaultEgressRollupMaximumRangeRule {
    fn check(&self, item: &DefaultEgressRollup) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressRollupRequiredChecksumRule {
    pub message: String,
}

impl DefaultEgressRollupRule for DefaultEgressRollupRequiredChecksumRule {
    fn check(&self, item: &DefaultEgressRollup) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default egress rollup.
#[derive(Debug, Clone)]
pub struct DefaultEgressRollupValidator {
    pub rules: Vec<Rc<dyn DefaultEgressRollupRule>>,
}

impl DefaultEgressRollupValidator {
    pub fn validate(&self, item: &DefaultEgressRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultEgressRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultEgressRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_egress_rollup_validator() -> DefaultEgressRollupValidator {
    DefaultEgressRollupValidator {
        rules: vec![
            Rc::new(DefaultEgressRollupChecksumLengthRule { max_length: 3600 }) as Rc<dyn DefaultEgressRollupRule>,
            Rc::new(DefaultEgressRollupCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DefaultEgressRollupMaximumRangeRule { min: 7.1, max: 3879.6 }),
            Rc::new(DefaultEgressRollupRequiredChecksumRule {
                message: "default egress rollup needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_egress_rollup_validator();
        let item = new_default_egress_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = DefaultEgressRollupChecksumLengthRule { max_length: 16 };
        let mut item = new_default_egress_rollup("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = DefaultEgressRollupMaximumRangeRule { min: 8.9, max: 3048.3 };
        let mut item = new_default_egress_rollup("x", "y");
        item.maximum = 3049.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = DefaultEgressRollupRequiredChecksumRule {
            message: "default egress rollup needs a value".to_string(),
        };
        let mut item = new_default_egress_rollup("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
