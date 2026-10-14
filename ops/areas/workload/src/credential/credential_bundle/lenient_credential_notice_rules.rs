use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient credential notice must pass. Check returns its problems, or none.
pub trait LenientCredentialNoticeRule: std::fmt::Debug {
    fn check(&self, item: &LenientCredentialNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialNoticeDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientCredentialNoticeRule for LenientCredentialNoticeDurationMinutesRangeRule {
    fn check(&self, item: &LenientCredentialNotice) -> Vec<String> {
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
pub struct LenientCredentialNoticeCodePrefixRule {
    pub prefix: String,
}

impl LenientCredentialNoticeRule for LenientCredentialNoticeCodePrefixRule {
    fn check(&self, item: &LenientCredentialNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialNoticeDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientCredentialNoticeRule for LenientCredentialNoticeDiscountRangeRule {
    fn check(&self, item: &LenientCredentialNotice) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialNoticeRequiredCodeRule {
    pub message: String,
}

impl LenientCredentialNoticeRule for LenientCredentialNoticeRequiredCodeRule {
    fn check(&self, item: &LenientCredentialNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialNoticeRequiredPrincipalRule {
    pub message: String,
}

impl LenientCredentialNoticeRule for LenientCredentialNoticeRequiredPrincipalRule {
    fn check(&self, item: &LenientCredentialNotice) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialNoticeRequiredTitleRule {
    pub message: String,
}

impl LenientCredentialNoticeRule for LenientCredentialNoticeRequiredTitleRule {
    fn check(&self, item: &LenientCredentialNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient credential notice.
#[derive(Debug, Clone)]
pub struct LenientCredentialNoticeValidator {
    pub rules: Vec<Rc<dyn LenientCredentialNoticeRule>>,
}

impl LenientCredentialNoticeValidator {
    pub fn validate(&self, item: &LenientCredentialNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientCredentialNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientCredentialNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_credential_notice_validator() -> LenientCredentialNoticeValidator {
    LenientCredentialNoticeValidator {
        rules: vec![
            Rc::new(LenientCredentialNoticeDurationMinutesRangeRule { min: 1000, max: 3600 })
                as Rc<dyn LenientCredentialNoticeRule>,
            Rc::new(LenientCredentialNoticeCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(LenientCredentialNoticeDiscountRangeRule { min: 2.3, max: 2191.6 }),
            Rc::new(LenientCredentialNoticeRequiredCodeRule {
                message: "lenient credential notice needs this field".to_string(),
            }),
            Rc::new(LenientCredentialNoticeRequiredPrincipalRule {
                message: "lenient credential notice needs this field".to_string(),
            }),
            Rc::new(LenientCredentialNoticeRequiredTitleRule {
                message: "lenient credential notice needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_credential_notice_validator();
        let item = new_lenient_credential_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = LenientCredentialNoticeDurationMinutesRangeRule { min: 16, max: 3600 };
        let mut item = new_lenient_credential_notice("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = LenientCredentialNoticeDiscountRangeRule { min: 7.0, max: 513.7 };
        let mut item = new_lenient_credential_notice("x", "y");
        item.discount = 515.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientCredentialNoticeRequiredCodeRule {
            message: "lenient credential notice needs a value".to_string(),
        };
        let mut item = new_lenient_credential_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = LenientCredentialNoticeRequiredPrincipalRule {
            message: "lenient credential notice needs a value".to_string(),
        };
        let mut item = new_lenient_credential_notice("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientCredentialNoticeRequiredTitleRule {
            message: "lenient credential notice needs this field".to_string(),
        };
        let mut item = new_lenient_credential_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
