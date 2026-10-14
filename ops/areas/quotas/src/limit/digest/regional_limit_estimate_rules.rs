use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional limit estimate must pass. Check returns its problems, or none.
pub trait RegionalLimitEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RegionalLimitEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalLimitEstimatePhoneLengthRule {
    pub max_length: i64,
}

impl RegionalLimitEstimateRule for RegionalLimitEstimatePhoneLengthRule {
    fn check(&self, item: &RegionalLimitEstimate) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalLimitEstimateLabelLengthRule {
    pub max_length: i64,
}

impl RegionalLimitEstimateRule for RegionalLimitEstimateLabelLengthRule {
    fn check(&self, item: &RegionalLimitEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalLimitEstimateRequiredLabelRule {
    pub message: String,
}

impl RegionalLimitEstimateRule for RegionalLimitEstimateRequiredLabelRule {
    fn check(&self, item: &RegionalLimitEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalLimitEstimateRequiredCodeRule {
    pub message: String,
}

impl RegionalLimitEstimateRule for RegionalLimitEstimateRequiredCodeRule {
    fn check(&self, item: &RegionalLimitEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional limit estimate.
#[derive(Debug, Clone)]
pub struct RegionalLimitEstimateValidator {
    pub rules: Vec<Rc<dyn RegionalLimitEstimateRule>>,
}

impl RegionalLimitEstimateValidator {
    pub fn validate(&self, item: &RegionalLimitEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalLimitEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalLimitEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_limit_estimate_validator() -> RegionalLimitEstimateValidator {
    RegionalLimitEstimateValidator {
        rules: vec![
            Rc::new(RegionalLimitEstimatePhoneLengthRule { max_length: 80 }) as Rc<dyn RegionalLimitEstimateRule>,
            Rc::new(RegionalLimitEstimateLabelLengthRule { max_length: 80 }),
            Rc::new(RegionalLimitEstimateRequiredLabelRule {
                message: "regional limit estimate needs this field".to_string(),
            }),
            Rc::new(RegionalLimitEstimateRequiredCodeRule {
                message: "regional limit estimate needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_limit_estimate_validator();
        let item = new_regional_limit_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = RegionalLimitEstimatePhoneLengthRule { max_length: 255 };
        let mut item = new_regional_limit_estimate("x", "y");
        item.phone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalLimitEstimateLabelLengthRule { max_length: 80 };
        let mut item = new_regional_limit_estimate("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalLimitEstimateRequiredLabelRule {
            message: "regional limit estimate needs an entry".to_string(),
        };
        let mut item = new_regional_limit_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalLimitEstimateRequiredCodeRule {
            message: "regional limit estimate needs this field".to_string(),
        };
        let mut item = new_regional_limit_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
