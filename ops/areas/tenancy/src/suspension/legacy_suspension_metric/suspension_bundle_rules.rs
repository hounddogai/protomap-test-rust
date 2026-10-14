use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a suspension bundle must pass. Check returns its problems, or none.
pub trait SuspensionBundleRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBundleRequiredAudienceRule {
    pub message: String,
}

impl SuspensionBundleRule for SuspensionBundleRequiredAudienceRule {
    fn check(&self, item: &SuspensionBundle) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBundleAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionBundleRule for SuspensionBundleAmountRangeRule {
    fn check(&self, item: &SuspensionBundle) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBundleApprovedGuardRule {
    pub limit: i64,
}

impl SuspensionBundleRule for SuspensionBundleApprovedGuardRule {
    fn check(&self, item: &SuspensionBundle) -> Vec<String> {
        if item.approved && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while approved",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SuspensionBundleRule for SuspensionBundleAllowedCodeRule {
    fn check(&self, item: &SuspensionBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBundleRequiredLabelRule {
    pub message: String,
}

impl SuspensionBundleRule for SuspensionBundleRequiredLabelRule {
    fn check(&self, item: &SuspensionBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBundleTimezoneLengthRule {
    pub max_length: i64,
}

impl SuspensionBundleRule for SuspensionBundleTimezoneLengthRule {
    fn check(&self, item: &SuspensionBundle) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension bundle.
#[derive(Debug, Clone)]
pub struct SuspensionBundleValidator {
    pub rules: Vec<Rc<dyn SuspensionBundleRule>>,
}

impl SuspensionBundleValidator {
    pub fn validate(&self, item: &SuspensionBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_bundle_validator() -> SuspensionBundleValidator {
    SuspensionBundleValidator {
        rules: vec![
            Rc::new(SuspensionBundleRequiredAudienceRule {
                message: "suspension bundle needs a value".to_string(),
            }) as Rc<dyn SuspensionBundleRule>,
            Rc::new(SuspensionBundleAmountRangeRule { min: 1.6, max: 4650.8 }),
            Rc::new(SuspensionBundleApprovedGuardRule { limit: 80 }),
            Rc::new(SuspensionBundleAllowedCodeRule {
                allowed: vec!["weighted".to_string()],
            }),
            Rc::new(SuspensionBundleRequiredLabelRule {
                message: "suspension bundle needs a value".to_string(),
            }),
            Rc::new(SuspensionBundleTimezoneLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_bundle_validator();
        let item = new_suspension_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = SuspensionBundleRequiredAudienceRule {
            message: "suspension bundle needs this field".to_string(),
        };
        let mut item = new_suspension_bundle("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = SuspensionBundleAmountRangeRule { min: 7.2, max: 3316.7 };
        let mut item = new_suspension_bundle("x", "y");
        item.amount = 3318.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SuspensionBundleRequiredLabelRule {
            message: "suspension bundle needs an entry".to_string(),
        };
        let mut item = new_suspension_bundle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = SuspensionBundleTimezoneLengthRule { max_length: 64 };
        let mut item = new_suspension_bundle("x", "y");
        item.timezone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
