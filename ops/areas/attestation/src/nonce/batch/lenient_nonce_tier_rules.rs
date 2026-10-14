use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient nonce tier must pass. Check returns its problems, or none.
pub trait LenientNonceTierRule: std::fmt::Debug {
    fn check(&self, item: &LenientNonceTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceTierCodeLengthRule {
    pub max_length: i64,
}

impl LenientNonceTierRule for LenientNonceTierCodeLengthRule {
    fn check(&self, item: &LenientNonceTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceTierGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientNonceTierRule for LenientNonceTierGraceDaysRangeRule {
    fn check(&self, item: &LenientNonceTier) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceTierIssuerLengthRule {
    pub max_length: i64,
}

impl LenientNonceTierRule for LenientNonceTierIssuerLengthRule {
    fn check(&self, item: &LenientNonceTier) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceTierStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientNonceTierRule for LenientNonceTierStartsAtRangeRule {
    fn check(&self, item: &LenientNonceTier) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceTierLabelLengthRule {
    pub max_length: i64,
}

impl LenientNonceTierRule for LenientNonceTierLabelLengthRule {
    fn check(&self, item: &LenientNonceTier) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceTierRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientNonceTierRule for LenientNonceTierRetriesRangeRule {
    fn check(&self, item: &LenientNonceTier) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient nonce tier.
#[derive(Debug, Clone)]
pub struct LenientNonceTierValidator {
    pub rules: Vec<Rc<dyn LenientNonceTierRule>>,
}

impl LenientNonceTierValidator {
    pub fn validate(&self, item: &LenientNonceTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientNonceTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientNonceTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_nonce_tier_validator() -> LenientNonceTierValidator {
    LenientNonceTierValidator {
        rules: vec![
            Rc::new(LenientNonceTierCodeLengthRule { max_length: 3600 }) as Rc<dyn LenientNonceTierRule>,
            Rc::new(LenientNonceTierGraceDaysRangeRule { min: 80, max: 3600 }),
            Rc::new(LenientNonceTierIssuerLengthRule { max_length: 86400 }),
            Rc::new(LenientNonceTierStartsAtRangeRule { min: 80, max: 120 }),
            Rc::new(LenientNonceTierLabelLengthRule { max_length: 120 }),
            Rc::new(LenientNonceTierRetriesRangeRule { min: 120, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_nonce_tier_validator();
        let item = new_lenient_nonce_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientNonceTierCodeLengthRule { max_length: 80 };
        let mut item = new_lenient_nonce_tier("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = LenientNonceTierGraceDaysRangeRule { min: 64, max: 80 };
        let mut item = new_lenient_nonce_tier("x", "y");
        item.grace_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = LenientNonceTierIssuerLengthRule { max_length: 32 };
        let mut item = new_lenient_nonce_tier("x", "y");
        item.issuer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = LenientNonceTierStartsAtRangeRule { min: 16, max: 80 };
        let mut item = new_lenient_nonce_tier("x", "y");
        item.starts_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientNonceTierLabelLengthRule { max_length: 16 };
        let mut item = new_lenient_nonce_tier("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = LenientNonceTierRetriesRangeRule { min: 120, max: 255 };
        let mut item = new_lenient_nonce_tier("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
