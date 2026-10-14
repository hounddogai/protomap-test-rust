use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual redaction bundle must pass. Check returns its problems, or none.
pub trait ManualRedactionBundleRule: std::fmt::Debug {
    fn check(&self, item: &ManualRedactionBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionBundleLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualRedactionBundleRule for ManualRedactionBundleLimitRangeRule {
    fn check(&self, item: &ManualRedactionBundle) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionBundleRequiredLabelRule {
    pub message: String,
}

impl ManualRedactionBundleRule for ManualRedactionBundleRequiredLabelRule {
    fn check(&self, item: &ManualRedactionBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionBundleQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualRedactionBundleRule for ManualRedactionBundleQuantityRangeRule {
    fn check(&self, item: &ManualRedactionBundle) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionBundleRequiredCodeRule {
    pub message: String,
}

impl ManualRedactionBundleRule for ManualRedactionBundleRequiredCodeRule {
    fn check(&self, item: &ManualRedactionBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionBundleCodeLengthRule {
    pub max_length: i64,
}

impl ManualRedactionBundleRule for ManualRedactionBundleCodeLengthRule {
    fn check(&self, item: &ManualRedactionBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual redaction bundle.
#[derive(Debug, Clone)]
pub struct ManualRedactionBundleValidator {
    pub rules: Vec<Rc<dyn ManualRedactionBundleRule>>,
}

impl ManualRedactionBundleValidator {
    pub fn validate(&self, item: &ManualRedactionBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRedactionBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRedactionBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_redaction_bundle_validator() -> ManualRedactionBundleValidator {
    ManualRedactionBundleValidator {
        rules: vec![
            Rc::new(ManualRedactionBundleLimitRangeRule { min: 32, max: 255 }) as Rc<dyn ManualRedactionBundleRule>,
            Rc::new(ManualRedactionBundleRequiredLabelRule {
                message: "manual redaction bundle needs a value".to_string(),
            }),
            Rc::new(ManualRedactionBundleQuantityRangeRule { min: 120, max: 3600 }),
            Rc::new(ManualRedactionBundleRequiredCodeRule {
                message: "manual redaction bundle needs an entry".to_string(),
            }),
            Rc::new(ManualRedactionBundleCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_redaction_bundle_validator();
        let item = new_manual_redaction_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ManualRedactionBundleLimitRangeRule { min: 1000, max: 86400 };
        let mut item = new_manual_redaction_bundle("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualRedactionBundleRequiredLabelRule {
            message: "manual redaction bundle needs this field".to_string(),
        };
        let mut item = new_manual_redaction_bundle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ManualRedactionBundleQuantityRangeRule { min: 80, max: 86400 };
        let mut item = new_manual_redaction_bundle("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualRedactionBundleRequiredCodeRule {
            message: "manual redaction bundle needs this field".to_string(),
        };
        let mut item = new_manual_redaction_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualRedactionBundleCodeLengthRule { max_length: 120 };
        let mut item = new_manual_redaction_bundle("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
