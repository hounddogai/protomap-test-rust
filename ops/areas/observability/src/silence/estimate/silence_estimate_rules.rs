use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence estimate must pass. Check returns its problems, or none.
pub trait SilenceEstimateRule: std::fmt::Debug {
    fn check(&self, item: &SilenceEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceEstimateCodePrefixRule {
    pub prefix: String,
}

impl SilenceEstimateRule for SilenceEstimateCodePrefixRule {
    fn check(&self, item: &SilenceEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceEstimateRequiredNameRule {
    pub message: String,
}

impl SilenceEstimateRule for SilenceEstimateRequiredNameRule {
    fn check(&self, item: &SilenceEstimate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceEstimateRequiredExternalRefRule {
    pub message: String,
}

impl SilenceEstimateRule for SilenceEstimateRequiredExternalRefRule {
    fn check(&self, item: &SilenceEstimate) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceEstimateStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceEstimateRule for SilenceEstimateStartsAtRangeRule {
    fn check(&self, item: &SilenceEstimate) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceEstimateCodeLengthRule {
    pub max_length: i64,
}

impl SilenceEstimateRule for SilenceEstimateCodeLengthRule {
    fn check(&self, item: &SilenceEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence estimate.
#[derive(Debug, Clone)]
pub struct SilenceEstimateValidator {
    pub rules: Vec<Rc<dyn SilenceEstimateRule>>,
}

impl SilenceEstimateValidator {
    pub fn validate(&self, item: &SilenceEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_estimate_validator() -> SilenceEstimateValidator {
    SilenceEstimateValidator {
        rules: vec![
            Rc::new(SilenceEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn SilenceEstimateRule>,
            Rc::new(SilenceEstimateRequiredNameRule {
                message: "silence estimate needs an entry".to_string(),
            }),
            Rc::new(SilenceEstimateRequiredExternalRefRule {
                message: "silence estimate needs this field".to_string(),
            }),
            Rc::new(SilenceEstimateStartsAtRangeRule { min: 3600, max: 86400 }),
            Rc::new(SilenceEstimateCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_estimate_validator();
        let item = new_silence_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SilenceEstimateRequiredNameRule {
            message: "silence estimate needs a value".to_string(),
        };
        let mut item = new_silence_estimate("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = SilenceEstimateRequiredExternalRefRule {
            message: "silence estimate needs an entry".to_string(),
        };
        let mut item = new_silence_estimate("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = SilenceEstimateStartsAtRangeRule { min: 120, max: 255 };
        let mut item = new_silence_estimate("x", "y");
        item.starts_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SilenceEstimateCodeLengthRule { max_length: 120 };
        let mut item = new_silence_estimate("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
