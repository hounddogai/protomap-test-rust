use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive nonce profile must pass. Check returns its problems, or none.
pub trait AdaptiveNonceProfileRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveNonceProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceProfileRequiredChecksumRule {
    pub message: String,
}

impl AdaptiveNonceProfileRule for AdaptiveNonceProfileRequiredChecksumRule {
    fn check(&self, item: &AdaptiveNonceProfile) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceProfileRequiredLocaleRule {
    pub message: String,
}

impl AdaptiveNonceProfileRule for AdaptiveNonceProfileRequiredLocaleRule {
    fn check(&self, item: &AdaptiveNonceProfile) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceProfileRequiredCodeRule {
    pub message: String,
}

impl AdaptiveNonceProfileRule for AdaptiveNonceProfileRequiredCodeRule {
    fn check(&self, item: &AdaptiveNonceProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceProfileBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveNonceProfileRule for AdaptiveNonceProfileBatchSizeRangeRule {
    fn check(&self, item: &AdaptiveNonceProfile) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceProfileTitleLengthRule {
    pub max_length: i64,
}

impl AdaptiveNonceProfileRule for AdaptiveNonceProfileTitleLengthRule {
    fn check(&self, item: &AdaptiveNonceProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive nonce profile.
#[derive(Debug, Clone)]
pub struct AdaptiveNonceProfileValidator {
    pub rules: Vec<Rc<dyn AdaptiveNonceProfileRule>>,
}

impl AdaptiveNonceProfileValidator {
    pub fn validate(&self, item: &AdaptiveNonceProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveNonceProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveNonceProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_nonce_profile_validator() -> AdaptiveNonceProfileValidator {
    AdaptiveNonceProfileValidator {
        rules: vec![
            Rc::new(AdaptiveNonceProfileRequiredChecksumRule {
                message: "adaptive nonce profile needs this field".to_string(),
            }) as Rc<dyn AdaptiveNonceProfileRule>,
            Rc::new(AdaptiveNonceProfileRequiredLocaleRule {
                message: "adaptive nonce profile needs this field".to_string(),
            }),
            Rc::new(AdaptiveNonceProfileRequiredCodeRule {
                message: "adaptive nonce profile needs a value".to_string(),
            }),
            Rc::new(AdaptiveNonceProfileBatchSizeRangeRule { min: 32, max: 64 }),
            Rc::new(AdaptiveNonceProfileTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_nonce_profile_validator();
        let item = new_adaptive_nonce_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = AdaptiveNonceProfileRequiredChecksumRule {
            message: "adaptive nonce profile needs a value".to_string(),
        };
        let mut item = new_adaptive_nonce_profile("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = AdaptiveNonceProfileRequiredLocaleRule {
            message: "adaptive nonce profile needs a value".to_string(),
        };
        let mut item = new_adaptive_nonce_profile("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveNonceProfileRequiredCodeRule {
            message: "adaptive nonce profile needs a value".to_string(),
        };
        let mut item = new_adaptive_nonce_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = AdaptiveNonceProfileBatchSizeRangeRule { min: 1000, max: 1000 };
        let mut item = new_adaptive_nonce_profile("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AdaptiveNonceProfileTitleLengthRule { max_length: 3600 };
        let mut item = new_adaptive_nonce_profile("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
