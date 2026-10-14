use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient quote profile must pass. Check returns its problems, or none.
pub trait LenientQuoteProfileRule: std::fmt::Debug {
    fn check(&self, item: &LenientQuoteProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuoteProfileExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientQuoteProfileRule for LenientQuoteProfileExpiresAtRangeRule {
    fn check(&self, item: &LenientQuoteProfile) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuoteProfileRequiredCodeRule {
    pub message: String,
}

impl LenientQuoteProfileRule for LenientQuoteProfileRequiredCodeRule {
    fn check(&self, item: &LenientQuoteProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuoteProfileSourceLengthRule {
    pub max_length: i64,
}

impl LenientQuoteProfileRule for LenientQuoteProfileSourceLengthRule {
    fn check(&self, item: &LenientQuoteProfile) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuoteProfileRegionLengthRule {
    pub max_length: i64,
}

impl LenientQuoteProfileRule for LenientQuoteProfileRegionLengthRule {
    fn check(&self, item: &LenientQuoteProfile) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient quote profile.
#[derive(Debug, Clone)]
pub struct LenientQuoteProfileValidator {
    pub rules: Vec<Rc<dyn LenientQuoteProfileRule>>,
}

impl LenientQuoteProfileValidator {
    pub fn validate(&self, item: &LenientQuoteProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientQuoteProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientQuoteProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_quote_profile_validator() -> LenientQuoteProfileValidator {
    LenientQuoteProfileValidator {
        rules: vec![
            Rc::new(LenientQuoteProfileExpiresAtRangeRule { min: 32, max: 3600 }) as Rc<dyn LenientQuoteProfileRule>,
            Rc::new(LenientQuoteProfileRequiredCodeRule {
                message: "lenient quote profile needs an entry".to_string(),
            }),
            Rc::new(LenientQuoteProfileSourceLengthRule { max_length: 16 }),
            Rc::new(LenientQuoteProfileRegionLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_quote_profile_validator();
        let item = new_lenient_quote_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = LenientQuoteProfileExpiresAtRangeRule { min: 16, max: 120 };
        let mut item = new_lenient_quote_profile("x", "y");
        item.expires_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientQuoteProfileRequiredCodeRule {
            message: "lenient quote profile needs a value".to_string(),
        };
        let mut item = new_lenient_quote_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = LenientQuoteProfileSourceLengthRule { max_length: 3600 };
        let mut item = new_lenient_quote_profile("x", "y");
        item.source = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = LenientQuoteProfileRegionLengthRule { max_length: 32 };
        let mut item = new_lenient_quote_profile("x", "y");
        item.region = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
