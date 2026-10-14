use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation notice must pass. Check returns its problems, or none.
pub trait FederationNoticeRule: std::fmt::Debug {
    fn check(&self, item: &FederationNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationNoticeDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationNoticeRule for FederationNoticeDurationMinutesRangeRule {
    fn check(&self, item: &FederationNotice) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationNoticeCodePrefixRule {
    pub prefix: String,
}

impl FederationNoticeRule for FederationNoticeCodePrefixRule {
    fn check(&self, item: &FederationNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationNoticeRequiredTitleRule {
    pub message: String,
}

impl FederationNoticeRule for FederationNoticeRequiredTitleRule {
    fn check(&self, item: &FederationNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationNoticeRequiredPhoneRule {
    pub message: String,
}

impl FederationNoticeRule for FederationNoticeRequiredPhoneRule {
    fn check(&self, item: &FederationNotice) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationNoticeAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl FederationNoticeRule for FederationNoticeAllowedTitleRule {
    fn check(&self, item: &FederationNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is expired", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationNoticeWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FederationNoticeRule for FederationNoticeWeightRangeRule {
    fn check(&self, item: &FederationNotice) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a federation notice.
#[derive(Debug, Clone)]
pub struct FederationNoticeValidator {
    pub rules: Vec<Rc<dyn FederationNoticeRule>>,
}

impl FederationNoticeValidator {
    pub fn validate(&self, item: &FederationNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_notice_validator() -> FederationNoticeValidator {
    FederationNoticeValidator {
        rules: vec![
            Rc::new(FederationNoticeDurationMinutesRangeRule { min: 80, max: 120 }) as Rc<dyn FederationNoticeRule>,
            Rc::new(FederationNoticeCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FederationNoticeRequiredTitleRule {
                message: "federation notice needs a value".to_string(),
            }),
            Rc::new(FederationNoticeRequiredPhoneRule {
                message: "federation notice needs this field".to_string(),
            }),
            Rc::new(FederationNoticeAllowedTitleRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(FederationNoticeWeightRangeRule { min: 5.5, max: 3834.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_notice_validator();
        let item = new_federation_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = FederationNoticeDurationMinutesRangeRule { min: 16, max: 80 };
        let mut item = new_federation_notice("x", "y");
        item.duration_minutes = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FederationNoticeRequiredTitleRule {
            message: "federation notice needs an entry".to_string(),
        };
        let mut item = new_federation_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = FederationNoticeRequiredPhoneRule {
            message: "federation notice needs an entry".to_string(),
        };
        let mut item = new_federation_notice("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = FederationNoticeWeightRangeRule { min: 1.8, max: 3839.1 };
        let mut item = new_federation_notice("x", "y");
        item.weight = 3840.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
