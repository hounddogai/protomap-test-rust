use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual postmortem profile must pass. Check returns its problems, or none.
pub trait ManualPostmortemProfileRule: std::fmt::Debug {
    fn check(&self, item: &ManualPostmortemProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPostmortemProfileQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualPostmortemProfileRule for ManualPostmortemProfileQuotaUnitsRangeRule {
    fn check(&self, item: &ManualPostmortemProfile) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPostmortemProfileRequiredSegmentRule {
    pub message: String,
}

impl ManualPostmortemProfileRule for ManualPostmortemProfileRequiredSegmentRule {
    fn check(&self, item: &ManualPostmortemProfile) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPostmortemProfileCodeLengthRule {
    pub max_length: i64,
}

impl ManualPostmortemProfileRule for ManualPostmortemProfileCodeLengthRule {
    fn check(&self, item: &ManualPostmortemProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPostmortemProfileRequiredLabelRule {
    pub message: String,
}

impl ManualPostmortemProfileRule for ManualPostmortemProfileRequiredLabelRule {
    fn check(&self, item: &ManualPostmortemProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPostmortemProfileAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ManualPostmortemProfileRule for ManualPostmortemProfileAllowedCodeRule {
    fn check(&self, item: &ManualPostmortemProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPostmortemProfileRequiredPhoneRule {
    pub message: String,
}

impl ManualPostmortemProfileRule for ManualPostmortemProfileRequiredPhoneRule {
    fn check(&self, item: &ManualPostmortemProfile) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual postmortem profile.
#[derive(Debug, Clone)]
pub struct ManualPostmortemProfileValidator {
    pub rules: Vec<Rc<dyn ManualPostmortemProfileRule>>,
}

impl ManualPostmortemProfileValidator {
    pub fn validate(&self, item: &ManualPostmortemProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualPostmortemProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualPostmortemProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_postmortem_profile_validator() -> ManualPostmortemProfileValidator {
    ManualPostmortemProfileValidator {
        rules: vec![
            Rc::new(ManualPostmortemProfileQuotaUnitsRangeRule { min: 16, max: 80 })
                as Rc<dyn ManualPostmortemProfileRule>,
            Rc::new(ManualPostmortemProfileRequiredSegmentRule {
                message: "manual postmortem profile needs a value".to_string(),
            }),
            Rc::new(ManualPostmortemProfileCodeLengthRule { max_length: 3600 }),
            Rc::new(ManualPostmortemProfileRequiredLabelRule {
                message: "manual postmortem profile needs an entry".to_string(),
            }),
            Rc::new(ManualPostmortemProfileAllowedCodeRule {
                allowed: vec!["scheduled".to_string()],
            }),
            Rc::new(ManualPostmortemProfileRequiredPhoneRule {
                message: "manual postmortem profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_postmortem_profile_validator();
        let item = new_manual_postmortem_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = ManualPostmortemProfileQuotaUnitsRangeRule { min: 1000, max: 3600 };
        let mut item = new_manual_postmortem_profile("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = ManualPostmortemProfileRequiredSegmentRule {
            message: "manual postmortem profile needs an entry".to_string(),
        };
        let mut item = new_manual_postmortem_profile("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualPostmortemProfileCodeLengthRule { max_length: 86400 };
        let mut item = new_manual_postmortem_profile("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualPostmortemProfileRequiredLabelRule {
            message: "manual postmortem profile needs a value".to_string(),
        };
        let mut item = new_manual_postmortem_profile("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ManualPostmortemProfileRequiredPhoneRule {
            message: "manual postmortem profile needs this field".to_string(),
        };
        let mut item = new_manual_postmortem_profile("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
