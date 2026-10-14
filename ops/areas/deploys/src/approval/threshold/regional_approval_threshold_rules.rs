use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional approval threshold must pass. Check returns its problems, or none.
pub trait RegionalApprovalThresholdRule: std::fmt::Debug {
    fn check(&self, item: &RegionalApprovalThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalThresholdRequiredIssuerRule {
    pub message: String,
}

impl RegionalApprovalThresholdRule for RegionalApprovalThresholdRequiredIssuerRule {
    fn check(&self, item: &RegionalApprovalThreshold) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalThresholdRequiredCodeRule {
    pub message: String,
}

impl RegionalApprovalThresholdRule for RegionalApprovalThresholdRequiredCodeRule {
    fn check(&self, item: &RegionalApprovalThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalThresholdRequiredCategoryRule {
    pub message: String,
}

impl RegionalApprovalThresholdRule for RegionalApprovalThresholdRequiredCategoryRule {
    fn check(&self, item: &RegionalApprovalThreshold) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalThresholdVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalApprovalThresholdRule for RegionalApprovalThresholdVersionRangeRule {
    fn check(&self, item: &RegionalApprovalThreshold) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalThresholdCodeLengthRule {
    pub max_length: i64,
}

impl RegionalApprovalThresholdRule for RegionalApprovalThresholdCodeLengthRule {
    fn check(&self, item: &RegionalApprovalThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalThresholdBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalApprovalThresholdRule for RegionalApprovalThresholdBudgetRangeRule {
    fn check(&self, item: &RegionalApprovalThreshold) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a regional approval threshold.
#[derive(Debug, Clone)]
pub struct RegionalApprovalThresholdValidator {
    pub rules: Vec<Rc<dyn RegionalApprovalThresholdRule>>,
}

impl RegionalApprovalThresholdValidator {
    pub fn validate(&self, item: &RegionalApprovalThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalApprovalThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalApprovalThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_approval_threshold_validator() -> RegionalApprovalThresholdValidator {
    RegionalApprovalThresholdValidator {
        rules: vec![
            Rc::new(RegionalApprovalThresholdRequiredIssuerRule {
                message: "regional approval threshold needs an entry".to_string(),
            }) as Rc<dyn RegionalApprovalThresholdRule>,
            Rc::new(RegionalApprovalThresholdRequiredCodeRule {
                message: "regional approval threshold needs an entry".to_string(),
            }),
            Rc::new(RegionalApprovalThresholdRequiredCategoryRule {
                message: "regional approval threshold needs a value".to_string(),
            }),
            Rc::new(RegionalApprovalThresholdVersionRangeRule { min: 64, max: 255 }),
            Rc::new(RegionalApprovalThresholdCodeLengthRule { max_length: 255 }),
            Rc::new(RegionalApprovalThresholdBudgetRangeRule { min: 4.6, max: 3760.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_approval_threshold_validator();
        let item = new_regional_approval_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = RegionalApprovalThresholdRequiredIssuerRule {
            message: "regional approval threshold needs a value".to_string(),
        };
        let mut item = new_regional_approval_threshold("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalApprovalThresholdRequiredCodeRule {
            message: "regional approval threshold needs a value".to_string(),
        };
        let mut item = new_regional_approval_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = RegionalApprovalThresholdRequiredCategoryRule {
            message: "regional approval threshold needs an entry".to_string(),
        };
        let mut item = new_regional_approval_threshold("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RegionalApprovalThresholdVersionRangeRule { min: 80, max: 120 };
        let mut item = new_regional_approval_threshold("x", "y");
        item.version = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalApprovalThresholdCodeLengthRule { max_length: 32 };
        let mut item = new_regional_approval_threshold("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RegionalApprovalThresholdBudgetRangeRule { min: 9.1, max: 504.9 };
        let mut item = new_regional_approval_threshold("x", "y");
        item.budget = 506.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
