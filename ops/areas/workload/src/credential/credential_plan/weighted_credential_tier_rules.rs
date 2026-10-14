use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted credential tier must pass. Check returns its problems, or none.
pub trait WeightedCredentialTierRule: std::fmt::Debug {
    fn check(&self, item: &WeightedCredentialTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialTierSourceLengthRule {
    pub max_length: i64,
}

impl WeightedCredentialTierRule for WeightedCredentialTierSourceLengthRule {
    fn check(&self, item: &WeightedCredentialTier) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialTierTitleLengthRule {
    pub max_length: i64,
}

impl WeightedCredentialTierRule for WeightedCredentialTierTitleLengthRule {
    fn check(&self, item: &WeightedCredentialTier) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialTierRequiredTitleRule {
    pub message: String,
}

impl WeightedCredentialTierRule for WeightedCredentialTierRequiredTitleRule {
    fn check(&self, item: &WeightedCredentialTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialTierApprovedGuardRule {
    pub limit: i64,
}

impl WeightedCredentialTierRule for WeightedCredentialTierApprovedGuardRule {
    fn check(&self, item: &WeightedCredentialTier) -> Vec<String> {
        if item.approved && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while approved",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted credential tier.
#[derive(Debug, Clone)]
pub struct WeightedCredentialTierValidator {
    pub rules: Vec<Rc<dyn WeightedCredentialTierRule>>,
}

impl WeightedCredentialTierValidator {
    pub fn validate(&self, item: &WeightedCredentialTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedCredentialTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedCredentialTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_credential_tier_validator() -> WeightedCredentialTierValidator {
    WeightedCredentialTierValidator {
        rules: vec![
            Rc::new(WeightedCredentialTierSourceLengthRule { max_length: 1000 }) as Rc<dyn WeightedCredentialTierRule>,
            Rc::new(WeightedCredentialTierTitleLengthRule { max_length: 64 }),
            Rc::new(WeightedCredentialTierRequiredTitleRule {
                message: "weighted credential tier needs an entry".to_string(),
            }),
            Rc::new(WeightedCredentialTierApprovedGuardRule { limit: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_credential_tier_validator();
        let item = new_weighted_credential_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = WeightedCredentialTierSourceLengthRule { max_length: 255 };
        let mut item = new_weighted_credential_tier("x", "y");
        item.source = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = WeightedCredentialTierTitleLengthRule { max_length: 16 };
        let mut item = new_weighted_credential_tier("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedCredentialTierRequiredTitleRule {
            message: "weighted credential tier needs this field".to_string(),
        };
        let mut item = new_weighted_credential_tier("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
