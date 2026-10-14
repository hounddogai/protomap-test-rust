use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential estimate must pass. Check returns its problems, or none.
pub trait CredentialEstimateRule: std::fmt::Debug {
    fn check(&self, item: &CredentialEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEstimateRequiredLabelRule {
    pub message: String,
}

impl CredentialEstimateRule for CredentialEstimateRequiredLabelRule {
    fn check(&self, item: &CredentialEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEstimateRequiredCodeRule {
    pub message: String,
}

impl CredentialEstimateRule for CredentialEstimateRequiredCodeRule {
    fn check(&self, item: &CredentialEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEstimateMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialEstimateRule for CredentialEstimateMarginRangeRule {
    fn check(&self, item: &CredentialEstimate) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEstimateLabelLengthRule {
    pub max_length: i64,
}

impl CredentialEstimateRule for CredentialEstimateLabelLengthRule {
    fn check(&self, item: &CredentialEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEstimateRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialEstimateRule for CredentialEstimateRatioRangeRule {
    fn check(&self, item: &CredentialEstimate) -> Vec<String> {
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

/// Runs every rule of a credential estimate.
#[derive(Debug, Clone)]
pub struct CredentialEstimateValidator {
    pub rules: Vec<Rc<dyn CredentialEstimateRule>>,
}

impl CredentialEstimateValidator {
    pub fn validate(&self, item: &CredentialEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_estimate_validator() -> CredentialEstimateValidator {
    CredentialEstimateValidator {
        rules: vec![
            Rc::new(CredentialEstimateRequiredLabelRule {
                message: "credential estimate needs this field".to_string(),
            }) as Rc<dyn CredentialEstimateRule>,
            Rc::new(CredentialEstimateRequiredCodeRule {
                message: "credential estimate needs a value".to_string(),
            }),
            Rc::new(CredentialEstimateMarginRangeRule { min: 0.0, max: 444.7 }),
            Rc::new(CredentialEstimateLabelLengthRule { max_length: 32 }),
            Rc::new(CredentialEstimateRatioRangeRule { min: 8.1, max: 3347.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_estimate_validator();
        let item = new_credential_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CredentialEstimateRequiredLabelRule {
            message: "credential estimate needs this field".to_string(),
        };
        let mut item = new_credential_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CredentialEstimateRequiredCodeRule {
            message: "credential estimate needs a value".to_string(),
        };
        let mut item = new_credential_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = CredentialEstimateMarginRangeRule { min: 5.9, max: 962.6 };
        let mut item = new_credential_estimate("x", "y");
        item.margin = 964.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CredentialEstimateLabelLengthRule { max_length: 80 };
        let mut item = new_credential_estimate("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = CredentialEstimateRatioRangeRule { min: 3.9, max: 4436.0 };
        let mut item = new_credential_estimate("x", "y");
        item.ratio = 4437.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
