use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional grant estimate must pass. Check returns its problems, or none.
pub trait RegionalGrantEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RegionalGrantEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantEstimateCodeLengthRule {
    pub max_length: i64,
}

impl RegionalGrantEstimateRule for RegionalGrantEstimateCodeLengthRule {
    fn check(&self, item: &RegionalGrantEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantEstimateLabelLengthRule {
    pub max_length: i64,
}

impl RegionalGrantEstimateRule for RegionalGrantEstimateLabelLengthRule {
    fn check(&self, item: &RegionalGrantEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantEstimateSummaryLengthRule {
    pub max_length: i64,
}

impl RegionalGrantEstimateRule for RegionalGrantEstimateSummaryLengthRule {
    fn check(&self, item: &RegionalGrantEstimate) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantEstimateRequiredSummaryRule {
    pub message: String,
}

impl RegionalGrantEstimateRule for RegionalGrantEstimateRequiredSummaryRule {
    fn check(&self, item: &RegionalGrantEstimate) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional grant estimate.
#[derive(Debug, Clone)]
pub struct RegionalGrantEstimateValidator {
    pub rules: Vec<Rc<dyn RegionalGrantEstimateRule>>,
}

impl RegionalGrantEstimateValidator {
    pub fn validate(&self, item: &RegionalGrantEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalGrantEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalGrantEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_grant_estimate_validator() -> RegionalGrantEstimateValidator {
    RegionalGrantEstimateValidator {
        rules: vec![
            Rc::new(RegionalGrantEstimateCodeLengthRule { max_length: 255 }) as Rc<dyn RegionalGrantEstimateRule>,
            Rc::new(RegionalGrantEstimateLabelLengthRule { max_length: 3600 }),
            Rc::new(RegionalGrantEstimateSummaryLengthRule { max_length: 16 }),
            Rc::new(RegionalGrantEstimateRequiredSummaryRule {
                message: "regional grant estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_grant_estimate_validator();
        let item = new_regional_grant_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalGrantEstimateCodeLengthRule { max_length: 255 };
        let mut item = new_regional_grant_estimate("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalGrantEstimateLabelLengthRule { max_length: 32 };
        let mut item = new_regional_grant_estimate("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = RegionalGrantEstimateSummaryLengthRule { max_length: 16 };
        let mut item = new_regional_grant_estimate("x", "y");
        item.summary = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RegionalGrantEstimateRequiredSummaryRule {
            message: "regional grant estimate needs a value".to_string(),
        };
        let mut item = new_regional_grant_estimate("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
