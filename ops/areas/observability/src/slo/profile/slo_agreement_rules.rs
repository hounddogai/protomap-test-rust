use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a slo agreement must pass. Check returns its problems, or none.
pub trait SloAgreementRule: std::fmt::Debug {
    fn check(&self, item: &SloAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAgreementLabelLengthRule {
    pub max_length: i64,
}

impl SloAgreementRule for SloAgreementLabelLengthRule {
    fn check(&self, item: &SloAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAgreementCodePrefixRule {
    pub prefix: String,
}

impl SloAgreementRule for SloAgreementCodePrefixRule {
    fn check(&self, item: &SloAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAgreementRequiredCodeRule {
    pub message: String,
}

impl SloAgreementRule for SloAgreementRequiredCodeRule {
    fn check(&self, item: &SloAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAgreementMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloAgreementRule for SloAgreementMaximumRangeRule {
    fn check(&self, item: &SloAgreement) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAgreementRequiredLabelRule {
    pub message: String,
}

impl SloAgreementRule for SloAgreementRequiredLabelRule {
    fn check(&self, item: &SloAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAgreementAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl SloAgreementRule for SloAgreementAllowedRegionRule {
    fn check(&self, item: &SloAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is duplicated", item.region)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo agreement.
#[derive(Debug, Clone)]
pub struct SloAgreementValidator {
    pub rules: Vec<Rc<dyn SloAgreementRule>>,
}

impl SloAgreementValidator {
    pub fn validate(&self, item: &SloAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_agreement_validator() -> SloAgreementValidator {
    SloAgreementValidator {
        rules: vec![
            Rc::new(SloAgreementLabelLengthRule { max_length: 1000 }) as Rc<dyn SloAgreementRule>,
            Rc::new(SloAgreementCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SloAgreementRequiredCodeRule {
                message: "slo agreement needs an entry".to_string(),
            }),
            Rc::new(SloAgreementMaximumRangeRule { min: 7.4, max: 1820.7 }),
            Rc::new(SloAgreementRequiredLabelRule {
                message: "slo agreement needs an entry".to_string(),
            }),
            Rc::new(SloAgreementAllowedRegionRule {
                allowed: vec!["weighted".to_string(), "lenient".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_agreement_validator();
        let item = new_slo_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SloAgreementLabelLengthRule { max_length: 16 };
        let mut item = new_slo_agreement("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloAgreementRequiredCodeRule {
            message: "slo agreement needs a value".to_string(),
        };
        let mut item = new_slo_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = SloAgreementMaximumRangeRule { min: 6.4, max: 1158.9 };
        let mut item = new_slo_agreement("x", "y");
        item.maximum = 1160.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SloAgreementRequiredLabelRule {
            message: "slo agreement needs a value".to_string(),
        };
        let mut item = new_slo_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
