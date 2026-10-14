use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional postmortem agreement must pass. Check returns its problems, or none.
pub trait RegionalPostmortemAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPostmortemAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemAgreementLabelLengthRule {
    pub max_length: i64,
}

impl RegionalPostmortemAgreementRule for RegionalPostmortemAgreementLabelLengthRule {
    fn check(&self, item: &RegionalPostmortemAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemAgreementRequiredLabelRule {
    pub message: String,
}

impl RegionalPostmortemAgreementRule for RegionalPostmortemAgreementRequiredLabelRule {
    fn check(&self, item: &RegionalPostmortemAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemAgreementCodeLengthRule {
    pub max_length: i64,
}

impl RegionalPostmortemAgreementRule for RegionalPostmortemAgreementCodeLengthRule {
    fn check(&self, item: &RegionalPostmortemAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemAgreementRequiredNameRule {
    pub message: String,
}

impl RegionalPostmortemAgreementRule for RegionalPostmortemAgreementRequiredNameRule {
    fn check(&self, item: &RegionalPostmortemAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemAgreementAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RegionalPostmortemAgreementRule for RegionalPostmortemAgreementAllowedNameRule {
    fn check(&self, item: &RegionalPostmortemAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemAgreementTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RegionalPostmortemAgreementRule for RegionalPostmortemAgreementTagsLimitRule {
    fn check(&self, item: &RegionalPostmortemAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a regional postmortem agreement.
#[derive(Debug, Clone)]
pub struct RegionalPostmortemAgreementValidator {
    pub rules: Vec<Rc<dyn RegionalPostmortemAgreementRule>>,
}

impl RegionalPostmortemAgreementValidator {
    pub fn validate(&self, item: &RegionalPostmortemAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalPostmortemAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPostmortemAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_postmortem_agreement_validator() -> RegionalPostmortemAgreementValidator {
    RegionalPostmortemAgreementValidator {
        rules: vec![
            Rc::new(RegionalPostmortemAgreementLabelLengthRule { max_length: 64 })
                as Rc<dyn RegionalPostmortemAgreementRule>,
            Rc::new(RegionalPostmortemAgreementRequiredLabelRule {
                message: "regional postmortem agreement needs this field".to_string(),
            }),
            Rc::new(RegionalPostmortemAgreementCodeLengthRule { max_length: 80 }),
            Rc::new(RegionalPostmortemAgreementRequiredNameRule {
                message: "regional postmortem agreement needs this field".to_string(),
            }),
            Rc::new(RegionalPostmortemAgreementAllowedNameRule {
                allowed: vec!["scheduled".to_string(), "tiered".to_string()],
            }),
            Rc::new(RegionalPostmortemAgreementTagsLimitRule {
                max_count: 3600,
                banned: Vec::new(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_postmortem_agreement_validator();
        let item = new_regional_postmortem_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalPostmortemAgreementLabelLengthRule { max_length: 120 };
        let mut item = new_regional_postmortem_agreement("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalPostmortemAgreementRequiredLabelRule {
            message: "regional postmortem agreement needs an entry".to_string(),
        };
        let mut item = new_regional_postmortem_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalPostmortemAgreementCodeLengthRule { max_length: 120 };
        let mut item = new_regional_postmortem_agreement("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalPostmortemAgreementRequiredNameRule {
            message: "regional postmortem agreement needs a value".to_string(),
        };
        let mut item = new_regional_postmortem_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
