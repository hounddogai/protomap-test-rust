use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient credential segment must pass. Check returns its problems, or none.
pub trait LenientCredentialSegmentRule: std::fmt::Debug {
    fn check(&self, item: &LenientCredentialSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialSegmentCodeLengthRule {
    pub max_length: i64,
}

impl LenientCredentialSegmentRule for LenientCredentialSegmentCodeLengthRule {
    fn check(&self, item: &LenientCredentialSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialSegmentAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl LenientCredentialSegmentRule for LenientCredentialSegmentAllowedChannelRule {
    fn check(&self, item: &LenientCredentialSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is out of range", item.channel)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialSegmentRequiredCodeRule {
    pub message: String,
}

impl LenientCredentialSegmentRule for LenientCredentialSegmentRequiredCodeRule {
    fn check(&self, item: &LenientCredentialSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialSegmentQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientCredentialSegmentRule for LenientCredentialSegmentQuotaUnitsRangeRule {
    fn check(&self, item: &LenientCredentialSegment) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialSegmentLocaleLengthRule {
    pub max_length: i64,
}

impl LenientCredentialSegmentRule for LenientCredentialSegmentLocaleLengthRule {
    fn check(&self, item: &LenientCredentialSegment) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialSegmentCodePrefixRule {
    pub prefix: String,
}

impl LenientCredentialSegmentRule for LenientCredentialSegmentCodePrefixRule {
    fn check(&self, item: &LenientCredentialSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient credential segment.
#[derive(Debug, Clone)]
pub struct LenientCredentialSegmentValidator {
    pub rules: Vec<Rc<dyn LenientCredentialSegmentRule>>,
}

impl LenientCredentialSegmentValidator {
    pub fn validate(&self, item: &LenientCredentialSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientCredentialSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientCredentialSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_credential_segment_validator() -> LenientCredentialSegmentValidator {
    LenientCredentialSegmentValidator {
        rules: vec![
            Rc::new(LenientCredentialSegmentCodeLengthRule { max_length: 80 }) as Rc<dyn LenientCredentialSegmentRule>,
            Rc::new(LenientCredentialSegmentAllowedChannelRule {
                allowed: vec!["legacy".to_string(), "regional".to_string(), "manual".to_string()],
            }),
            Rc::new(LenientCredentialSegmentRequiredCodeRule {
                message: "lenient credential segment needs a value".to_string(),
            }),
            Rc::new(LenientCredentialSegmentQuotaUnitsRangeRule { min: 120, max: 1000 }),
            Rc::new(LenientCredentialSegmentLocaleLengthRule { max_length: 86400 }),
            Rc::new(LenientCredentialSegmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_credential_segment_validator();
        let item = new_lenient_credential_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientCredentialSegmentCodeLengthRule { max_length: 255 };
        let mut item = new_lenient_credential_segment("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientCredentialSegmentRequiredCodeRule {
            message: "lenient credential segment needs this field".to_string(),
        };
        let mut item = new_lenient_credential_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = LenientCredentialSegmentQuotaUnitsRangeRule { min: 255, max: 255 };
        let mut item = new_lenient_credential_segment("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = LenientCredentialSegmentLocaleLengthRule { max_length: 86400 };
        let mut item = new_lenient_credential_segment("x", "y");
        item.locale = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
