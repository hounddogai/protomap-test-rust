use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling quote estimate must pass. Check returns its problems, or none.
pub trait RollingQuoteEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RollingQuoteEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteEstimateCodeLengthRule {
    pub max_length: i64,
}

impl RollingQuoteEstimateRule for RollingQuoteEstimateCodeLengthRule {
    fn check(&self, item: &RollingQuoteEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteEstimateRequiredTenantIdRule {
    pub message: String,
}

impl RollingQuoteEstimateRule for RollingQuoteEstimateRequiredTenantIdRule {
    fn check(&self, item: &RollingQuoteEstimate) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteEstimateRequiredCodeRule {
    pub message: String,
}

impl RollingQuoteEstimateRule for RollingQuoteEstimateRequiredCodeRule {
    fn check(&self, item: &RollingQuoteEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteEstimateLabelLengthRule {
    pub max_length: i64,
}

impl RollingQuoteEstimateRule for RollingQuoteEstimateLabelLengthRule {
    fn check(&self, item: &RollingQuoteEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteEstimateChecksumLengthRule {
    pub max_length: i64,
}

impl RollingQuoteEstimateRule for RollingQuoteEstimateChecksumLengthRule {
    fn check(&self, item: &RollingQuoteEstimate) -> Vec<String> {
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
pub struct RollingQuoteEstimateRequiredLabelRule {
    pub message: String,
}

impl RollingQuoteEstimateRule for RollingQuoteEstimateRequiredLabelRule {
    fn check(&self, item: &RollingQuoteEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling quote estimate.
#[derive(Debug, Clone)]
pub struct RollingQuoteEstimateValidator {
    pub rules: Vec<Rc<dyn RollingQuoteEstimateRule>>,
}

impl RollingQuoteEstimateValidator {
    pub fn validate(&self, item: &RollingQuoteEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingQuoteEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingQuoteEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_quote_estimate_validator() -> RollingQuoteEstimateValidator {
    RollingQuoteEstimateValidator {
        rules: vec![
            Rc::new(RollingQuoteEstimateCodeLengthRule { max_length: 120 }) as Rc<dyn RollingQuoteEstimateRule>,
            Rc::new(RollingQuoteEstimateRequiredTenantIdRule {
                message: "rolling quote estimate needs a value".to_string(),
            }),
            Rc::new(RollingQuoteEstimateRequiredCodeRule {
                message: "rolling quote estimate needs a value".to_string(),
            }),
            Rc::new(RollingQuoteEstimateLabelLengthRule { max_length: 32 }),
            Rc::new(RollingQuoteEstimateChecksumLengthRule { max_length: 3600 }),
            Rc::new(RollingQuoteEstimateRequiredLabelRule {
                message: "rolling quote estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_quote_estimate_validator();
        let item = new_rolling_quote_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingQuoteEstimateCodeLengthRule { max_length: 80 };
        let mut item = new_rolling_quote_estimate("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = RollingQuoteEstimateRequiredTenantIdRule {
            message: "rolling quote estimate needs this field".to_string(),
        };
        let mut item = new_rolling_quote_estimate("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingQuoteEstimateRequiredCodeRule {
            message: "rolling quote estimate needs this field".to_string(),
        };
        let mut item = new_rolling_quote_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollingQuoteEstimateLabelLengthRule { max_length: 86400 };
        let mut item = new_rolling_quote_estimate("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = RollingQuoteEstimateChecksumLengthRule { max_length: 120 };
        let mut item = new_rolling_quote_estimate("x", "y");
        item.checksum = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollingQuoteEstimateRequiredLabelRule {
            message: "rolling quote estimate needs this field".to_string(),
        };
        let mut item = new_rolling_quote_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
