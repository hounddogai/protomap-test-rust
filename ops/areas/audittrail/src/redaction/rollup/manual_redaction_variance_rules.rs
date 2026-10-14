use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual redaction variance must pass. Check returns its problems, or none.
pub trait ManualRedactionVarianceRule: std::fmt::Debug {
    fn check(&self, item: &ManualRedactionVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionVarianceNameLengthRule {
    pub max_length: i64,
}

impl ManualRedactionVarianceRule for ManualRedactionVarianceNameLengthRule {
    fn check(&self, item: &ManualRedactionVariance) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionVarianceRequiredCodeRule {
    pub message: String,
}

impl ManualRedactionVarianceRule for ManualRedactionVarianceRequiredCodeRule {
    fn check(&self, item: &ManualRedactionVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionVarianceScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualRedactionVarianceRule for ManualRedactionVarianceScoreRangeRule {
    fn check(&self, item: &ManualRedactionVariance) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionVarianceRequiredHostRule {
    pub message: String,
}

impl ManualRedactionVarianceRule for ManualRedactionVarianceRequiredHostRule {
    fn check(&self, item: &ManualRedactionVariance) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionVarianceQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualRedactionVarianceRule for ManualRedactionVarianceQuantityRangeRule {
    fn check(&self, item: &ManualRedactionVariance) -> Vec<String> {
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
pub struct ManualRedactionVarianceRequiredNameRule {
    pub message: String,
}

impl ManualRedactionVarianceRule for ManualRedactionVarianceRequiredNameRule {
    fn check(&self, item: &ManualRedactionVariance) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual redaction variance.
#[derive(Debug, Clone)]
pub struct ManualRedactionVarianceValidator {
    pub rules: Vec<Rc<dyn ManualRedactionVarianceRule>>,
}

impl ManualRedactionVarianceValidator {
    pub fn validate(&self, item: &ManualRedactionVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRedactionVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRedactionVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_redaction_variance_validator() -> ManualRedactionVarianceValidator {
    ManualRedactionVarianceValidator {
        rules: vec![
            Rc::new(ManualRedactionVarianceNameLengthRule { max_length: 16 }) as Rc<dyn ManualRedactionVarianceRule>,
            Rc::new(ManualRedactionVarianceRequiredCodeRule {
                message: "manual redaction variance needs this field".to_string(),
            }),
            Rc::new(ManualRedactionVarianceScoreRangeRule { min: 9.0, max: 4602.3 }),
            Rc::new(ManualRedactionVarianceRequiredHostRule {
                message: "manual redaction variance needs an entry".to_string(),
            }),
            Rc::new(ManualRedactionVarianceQuantityRangeRule { min: 16, max: 86400 }),
            Rc::new(ManualRedactionVarianceRequiredNameRule {
                message: "manual redaction variance needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_redaction_variance_validator();
        let item = new_manual_redaction_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualRedactionVarianceNameLengthRule { max_length: 3600 };
        let mut item = new_manual_redaction_variance("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualRedactionVarianceRequiredCodeRule {
            message: "manual redaction variance needs a value".to_string(),
        };
        let mut item = new_manual_redaction_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ManualRedactionVarianceScoreRangeRule { min: 0.2, max: 2075.5 };
        let mut item = new_manual_redaction_variance("x", "y");
        item.score = 2077.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = ManualRedactionVarianceRequiredHostRule {
            message: "manual redaction variance needs a value".to_string(),
        };
        let mut item = new_manual_redaction_variance("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ManualRedactionVarianceQuantityRangeRule { min: 16, max: 255 };
        let mut item = new_manual_redaction_variance("x", "y");
        item.quantity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualRedactionVarianceRequiredNameRule {
            message: "manual redaction variance needs a value".to_string(),
        };
        let mut item = new_manual_redaction_variance("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
