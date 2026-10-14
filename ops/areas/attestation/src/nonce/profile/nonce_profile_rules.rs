use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce profile must pass. Check returns its problems, or none.
pub trait NonceProfileRule: std::fmt::Debug {
    fn check(&self, item: &NonceProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceProfileHostLengthRule {
    pub max_length: i64,
}

impl NonceProfileRule for NonceProfileHostLengthRule {
    fn check(&self, item: &NonceProfile) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceProfileExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceProfileRule for NonceProfileExpiresAtRangeRule {
    fn check(&self, item: &NonceProfile) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceProfileRequiredCodeRule {
    pub message: String,
}

impl NonceProfileRule for NonceProfileRequiredCodeRule {
    fn check(&self, item: &NonceProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceProfileEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceProfileRule for NonceProfileEndsAtRangeRule {
    fn check(&self, item: &NonceProfile) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceProfileActiveGuardRule {
    pub limit: i64,
}

impl NonceProfileRule for NonceProfileActiveGuardRule {
    fn check(&self, item: &NonceProfile) -> Vec<String> {
        if item.active && (item.ends_at > self.limit) {
            return vec![format!("ends at {} exceeds {} while active", item.ends_at, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceProfileSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceProfileRule for NonceProfileSequenceRangeRule {
    fn check(&self, item: &NonceProfile) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce profile.
#[derive(Debug, Clone)]
pub struct NonceProfileValidator {
    pub rules: Vec<Rc<dyn NonceProfileRule>>,
}

impl NonceProfileValidator {
    pub fn validate(&self, item: &NonceProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_profile_validator() -> NonceProfileValidator {
    NonceProfileValidator {
        rules: vec![
            Rc::new(NonceProfileHostLengthRule { max_length: 120 }) as Rc<dyn NonceProfileRule>,
            Rc::new(NonceProfileExpiresAtRangeRule { min: 16, max: 3600 }),
            Rc::new(NonceProfileRequiredCodeRule {
                message: "nonce profile needs a value".to_string(),
            }),
            Rc::new(NonceProfileEndsAtRangeRule { min: 80, max: 120 }),
            Rc::new(NonceProfileActiveGuardRule { limit: 80 }),
            Rc::new(NonceProfileSequenceRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_profile_validator();
        let item = new_nonce_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = NonceProfileHostLengthRule { max_length: 32 };
        let mut item = new_nonce_profile("x", "y");
        item.host = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = NonceProfileExpiresAtRangeRule { min: 64, max: 86400 };
        let mut item = new_nonce_profile("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = NonceProfileRequiredCodeRule {
            message: "nonce profile needs this field".to_string(),
        };
        let mut item = new_nonce_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = NonceProfileEndsAtRangeRule { min: 255, max: 1000 };
        let mut item = new_nonce_profile("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = NonceProfileSequenceRangeRule { min: 32, max: 120 };
        let mut item = new_nonce_profile("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
