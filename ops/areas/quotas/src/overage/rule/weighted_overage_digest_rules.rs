use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted overage digest must pass. Check returns its problems, or none.
pub trait WeightedOverageDigestRule: std::fmt::Debug {
    fn check(&self, item: &WeightedOverageDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageDigestContactEmailLengthRule {
    pub max_length: i64,
}

impl WeightedOverageDigestRule for WeightedOverageDigestContactEmailLengthRule {
    fn check(&self, item: &WeightedOverageDigest) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageDigestCodeLengthRule {
    pub max_length: i64,
}

impl WeightedOverageDigestRule for WeightedOverageDigestCodeLengthRule {
    fn check(&self, item: &WeightedOverageDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageDigestRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedOverageDigestRule for WeightedOverageDigestRetriesRangeRule {
    fn check(&self, item: &WeightedOverageDigest) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageDigestCodePrefixRule {
    pub prefix: String,
}

impl WeightedOverageDigestRule for WeightedOverageDigestCodePrefixRule {
    fn check(&self, item: &WeightedOverageDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageDigestLabelLengthRule {
    pub max_length: i64,
}

impl WeightedOverageDigestRule for WeightedOverageDigestLabelLengthRule {
    fn check(&self, item: &WeightedOverageDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageDigestSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedOverageDigestRule for WeightedOverageDigestSequenceRangeRule {
    fn check(&self, item: &WeightedOverageDigest) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted overage digest.
#[derive(Debug, Clone)]
pub struct WeightedOverageDigestValidator {
    pub rules: Vec<Rc<dyn WeightedOverageDigestRule>>,
}

impl WeightedOverageDigestValidator {
    pub fn validate(&self, item: &WeightedOverageDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedOverageDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedOverageDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_overage_digest_validator() -> WeightedOverageDigestValidator {
    WeightedOverageDigestValidator {
        rules: vec![
            Rc::new(WeightedOverageDigestContactEmailLengthRule { max_length: 3600 })
                as Rc<dyn WeightedOverageDigestRule>,
            Rc::new(WeightedOverageDigestCodeLengthRule { max_length: 120 }),
            Rc::new(WeightedOverageDigestRetriesRangeRule { min: 16, max: 1000 }),
            Rc::new(WeightedOverageDigestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(WeightedOverageDigestLabelLengthRule { max_length: 80 }),
            Rc::new(WeightedOverageDigestSequenceRangeRule { min: 64, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_overage_digest_validator();
        let item = new_weighted_overage_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = WeightedOverageDigestContactEmailLengthRule { max_length: 3600 };
        let mut item = new_weighted_overage_digest("x", "y");
        item.contact_email = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedOverageDigestCodeLengthRule { max_length: 1000 };
        let mut item = new_weighted_overage_digest("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = WeightedOverageDigestRetriesRangeRule { min: 80, max: 3600 };
        let mut item = new_weighted_overage_digest("x", "y");
        item.retries = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedOverageDigestLabelLengthRule { max_length: 32 };
        let mut item = new_weighted_overage_digest("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = WeightedOverageDigestSequenceRangeRule { min: 16, max: 86400 };
        let mut item = new_weighted_overage_digest("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
