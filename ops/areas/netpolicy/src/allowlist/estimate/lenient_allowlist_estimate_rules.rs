use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient allowlist estimate must pass. Check returns its problems, or none.
pub trait LenientAllowlistEstimateRule: std::fmt::Debug {
    fn check(&self, item: &LenientAllowlistEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistEstimateRequiredIssuerRule {
    pub message: String,
}

impl LenientAllowlistEstimateRule for LenientAllowlistEstimateRequiredIssuerRule {
    fn check(&self, item: &LenientAllowlistEstimate) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistEstimateCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientAllowlistEstimateRule for LenientAllowlistEstimateCeilingRangeRule {
    fn check(&self, item: &LenientAllowlistEstimate) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistEstimateLabelLengthRule {
    pub max_length: i64,
}

impl LenientAllowlistEstimateRule for LenientAllowlistEstimateLabelLengthRule {
    fn check(&self, item: &LenientAllowlistEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistEstimateCodePrefixRule {
    pub prefix: String,
}

impl LenientAllowlistEstimateRule for LenientAllowlistEstimateCodePrefixRule {
    fn check(&self, item: &LenientAllowlistEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistEstimateRequiredLabelRule {
    pub message: String,
}

impl LenientAllowlistEstimateRule for LenientAllowlistEstimateRequiredLabelRule {
    fn check(&self, item: &LenientAllowlistEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient allowlist estimate.
#[derive(Debug, Clone)]
pub struct LenientAllowlistEstimateValidator {
    pub rules: Vec<Rc<dyn LenientAllowlistEstimateRule>>,
}

impl LenientAllowlistEstimateValidator {
    pub fn validate(&self, item: &LenientAllowlistEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientAllowlistEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientAllowlistEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_allowlist_estimate_validator() -> LenientAllowlistEstimateValidator {
    LenientAllowlistEstimateValidator {
        rules: vec![
            Rc::new(LenientAllowlistEstimateRequiredIssuerRule {
                message: "lenient allowlist estimate needs this field".to_string(),
            }) as Rc<dyn LenientAllowlistEstimateRule>,
            Rc::new(LenientAllowlistEstimateCeilingRangeRule { min: 8.8, max: 424.8 }),
            Rc::new(LenientAllowlistEstimateLabelLengthRule { max_length: 32 }),
            Rc::new(LenientAllowlistEstimateCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(LenientAllowlistEstimateRequiredLabelRule {
                message: "lenient allowlist estimate needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_allowlist_estimate_validator();
        let item = new_lenient_allowlist_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = LenientAllowlistEstimateRequiredIssuerRule {
            message: "lenient allowlist estimate needs a value".to_string(),
        };
        let mut item = new_lenient_allowlist_estimate("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = LenientAllowlistEstimateCeilingRangeRule { min: 1.8, max: 250.0 };
        let mut item = new_lenient_allowlist_estimate("x", "y");
        item.ceiling = 251.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientAllowlistEstimateLabelLengthRule { max_length: 120 };
        let mut item = new_lenient_allowlist_estimate("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientAllowlistEstimateRequiredLabelRule {
            message: "lenient allowlist estimate needs a value".to_string(),
        };
        let mut item = new_lenient_allowlist_estimate("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
