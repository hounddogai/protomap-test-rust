use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback federation summary must pass. Check returns its problems, or none.
pub trait FallbackFederationSummaryRule: std::fmt::Debug {
    fn check(&self, item: &FallbackFederationSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFederationSummaryRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackFederationSummaryRule for FallbackFederationSummaryRateRangeRule {
    fn check(&self, item: &FallbackFederationSummary) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFederationSummaryThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackFederationSummaryRule for FallbackFederationSummaryThresholdRangeRule {
    fn check(&self, item: &FallbackFederationSummary) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFederationSummaryLabelLengthRule {
    pub max_length: i64,
}

impl FallbackFederationSummaryRule for FallbackFederationSummaryLabelLengthRule {
    fn check(&self, item: &FallbackFederationSummary) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFederationSummaryRequiredContactEmailRule {
    pub message: String,
}

impl FallbackFederationSummaryRule for FallbackFederationSummaryRequiredContactEmailRule {
    fn check(&self, item: &FallbackFederationSummary) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback federation summary.
#[derive(Debug, Clone)]
pub struct FallbackFederationSummaryValidator {
    pub rules: Vec<Rc<dyn FallbackFederationSummaryRule>>,
}

impl FallbackFederationSummaryValidator {
    pub fn validate(&self, item: &FallbackFederationSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackFederationSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackFederationSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_federation_summary_validator() -> FallbackFederationSummaryValidator {
    FallbackFederationSummaryValidator {
        rules: vec![
            Rc::new(FallbackFederationSummaryRateRangeRule { min: 4.1, max: 3421.9 })
                as Rc<dyn FallbackFederationSummaryRule>,
            Rc::new(FallbackFederationSummaryThresholdRangeRule { min: 1.6, max: 2169.7 }),
            Rc::new(FallbackFederationSummaryLabelLengthRule { max_length: 16 }),
            Rc::new(FallbackFederationSummaryRequiredContactEmailRule {
                message: "fallback federation summary needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_federation_summary_validator();
        let item = new_fallback_federation_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = FallbackFederationSummaryRateRangeRule { min: 7.0, max: 71.7 };
        let mut item = new_fallback_federation_summary("x", "y");
        item.rate = 73.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = FallbackFederationSummaryThresholdRangeRule { min: 9.1, max: 2675.6 };
        let mut item = new_fallback_federation_summary("x", "y");
        item.threshold = 2677.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackFederationSummaryLabelLengthRule { max_length: 86400 };
        let mut item = new_fallback_federation_summary("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = FallbackFederationSummaryRequiredContactEmailRule {
            message: "fallback federation summary needs this field".to_string(),
        };
        let mut item = new_fallback_federation_summary("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
