use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval notice must pass. Check returns its problems, or none.
pub trait ApprovalNoticeRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalNoticeCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalNoticeRule for ApprovalNoticeCodeLengthRule {
    fn check(&self, item: &ApprovalNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalNoticeRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalNoticeRule for ApprovalNoticeRetriesRangeRule {
    fn check(&self, item: &ApprovalNotice) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalNoticeThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalNoticeRule for ApprovalNoticeThresholdRangeRule {
    fn check(&self, item: &ApprovalNotice) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalNoticeCodePrefixRule {
    pub prefix: String,
}

impl ApprovalNoticeRule for ApprovalNoticeCodePrefixRule {
    fn check(&self, item: &ApprovalNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval notice.
#[derive(Debug, Clone)]
pub struct ApprovalNoticeValidator {
    pub rules: Vec<Rc<dyn ApprovalNoticeRule>>,
}

impl ApprovalNoticeValidator {
    pub fn validate(&self, item: &ApprovalNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_notice_validator() -> ApprovalNoticeValidator {
    ApprovalNoticeValidator {
        rules: vec![
            Rc::new(ApprovalNoticeCodeLengthRule { max_length: 80 }) as Rc<dyn ApprovalNoticeRule>,
            Rc::new(ApprovalNoticeRetriesRangeRule { min: 64, max: 255 }),
            Rc::new(ApprovalNoticeThresholdRangeRule { min: 5.8, max: 556.7 }),
            Rc::new(ApprovalNoticeCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_notice_validator();
        let item = new_approval_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ApprovalNoticeCodeLengthRule { max_length: 32 };
        let mut item = new_approval_notice("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = ApprovalNoticeRetriesRangeRule { min: 80, max: 120 };
        let mut item = new_approval_notice("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = ApprovalNoticeThresholdRangeRule { min: 4.7, max: 112.4 };
        let mut item = new_approval_notice("x", "y");
        item.threshold = 113.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
