use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive credential tier must pass. Check returns its problems, or none.
pub trait AdaptiveCredentialTierRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveCredentialTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialTierRequiredNameRule {
    pub message: String,
}

impl AdaptiveCredentialTierRule for AdaptiveCredentialTierRequiredNameRule {
    fn check(&self, item: &AdaptiveCredentialTier) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialTierRequiredCodeRule {
    pub message: String,
}

impl AdaptiveCredentialTierRule for AdaptiveCredentialTierRequiredCodeRule {
    fn check(&self, item: &AdaptiveCredentialTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialTierRequiredTitleRule {
    pub message: String,
}

impl AdaptiveCredentialTierRule for AdaptiveCredentialTierRequiredTitleRule {
    fn check(&self, item: &AdaptiveCredentialTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive credential tier.
#[derive(Debug, Clone)]
pub struct AdaptiveCredentialTierValidator {
    pub rules: Vec<Rc<dyn AdaptiveCredentialTierRule>>,
}

impl AdaptiveCredentialTierValidator {
    pub fn validate(&self, item: &AdaptiveCredentialTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveCredentialTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveCredentialTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_credential_tier_validator() -> AdaptiveCredentialTierValidator {
    AdaptiveCredentialTierValidator {
        rules: vec![
            Rc::new(AdaptiveCredentialTierRequiredNameRule {
                message: "adaptive credential tier needs a value".to_string(),
            }) as Rc<dyn AdaptiveCredentialTierRule>,
            Rc::new(AdaptiveCredentialTierRequiredCodeRule {
                message: "adaptive credential tier needs this field".to_string(),
            }),
            Rc::new(AdaptiveCredentialTierRequiredTitleRule {
                message: "adaptive credential tier needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_credential_tier_validator();
        let item = new_adaptive_credential_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AdaptiveCredentialTierRequiredNameRule {
            message: "adaptive credential tier needs a value".to_string(),
        };
        let mut item = new_adaptive_credential_tier("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveCredentialTierRequiredCodeRule {
            message: "adaptive credential tier needs an entry".to_string(),
        };
        let mut item = new_adaptive_credential_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptiveCredentialTierRequiredTitleRule {
            message: "adaptive credential tier needs this field".to_string(),
        };
        let mut item = new_adaptive_credential_tier("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
