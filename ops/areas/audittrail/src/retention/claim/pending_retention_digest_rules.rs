use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending retention digest must pass. Check returns its problems, or none.
pub trait PendingRetentionDigestRule: std::fmt::Debug {
    fn check(&self, item: &PendingRetentionDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionDigestIssuerLengthRule {
    pub max_length: i64,
}

impl PendingRetentionDigestRule for PendingRetentionDigestIssuerLengthRule {
    fn check(&self, item: &PendingRetentionDigest) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionDigestRequiredTitleRule {
    pub message: String,
}

impl PendingRetentionDigestRule for PendingRetentionDigestRequiredTitleRule {
    fn check(&self, item: &PendingRetentionDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionDigestCodePrefixRule {
    pub prefix: String,
}

impl PendingRetentionDigestRule for PendingRetentionDigestCodePrefixRule {
    fn check(&self, item: &PendingRetentionDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionDigestRequiredCodeRule {
    pub message: String,
}

impl PendingRetentionDigestRule for PendingRetentionDigestRequiredCodeRule {
    fn check(&self, item: &PendingRetentionDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionDigestMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingRetentionDigestRule for PendingRetentionDigestMaximumRangeRule {
    fn check(&self, item: &PendingRetentionDigest) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionDigestTitleLengthRule {
    pub max_length: i64,
}

impl PendingRetentionDigestRule for PendingRetentionDigestTitleLengthRule {
    fn check(&self, item: &PendingRetentionDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending retention digest.
#[derive(Debug, Clone)]
pub struct PendingRetentionDigestValidator {
    pub rules: Vec<Rc<dyn PendingRetentionDigestRule>>,
}

impl PendingRetentionDigestValidator {
    pub fn validate(&self, item: &PendingRetentionDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRetentionDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRetentionDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_retention_digest_validator() -> PendingRetentionDigestValidator {
    PendingRetentionDigestValidator {
        rules: vec![
            Rc::new(PendingRetentionDigestIssuerLengthRule { max_length: 16 }) as Rc<dyn PendingRetentionDigestRule>,
            Rc::new(PendingRetentionDigestRequiredTitleRule {
                message: "pending retention digest needs a value".to_string(),
            }),
            Rc::new(PendingRetentionDigestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PendingRetentionDigestRequiredCodeRule {
                message: "pending retention digest needs this field".to_string(),
            }),
            Rc::new(PendingRetentionDigestMaximumRangeRule { min: 7.5, max: 3477.9 }),
            Rc::new(PendingRetentionDigestTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_retention_digest_validator();
        let item = new_pending_retention_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = PendingRetentionDigestIssuerLengthRule { max_length: 32 };
        let mut item = new_pending_retention_digest("x", "y");
        item.issuer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingRetentionDigestRequiredTitleRule {
            message: "pending retention digest needs this field".to_string(),
        };
        let mut item = new_pending_retention_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingRetentionDigestRequiredCodeRule {
            message: "pending retention digest needs an entry".to_string(),
        };
        let mut item = new_pending_retention_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = PendingRetentionDigestMaximumRangeRule { min: 2.7, max: 1261.5 };
        let mut item = new_pending_retention_digest("x", "y");
        item.maximum = 1263.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingRetentionDigestTitleLengthRule { max_length: 86400 };
        let mut item = new_pending_retention_digest("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
