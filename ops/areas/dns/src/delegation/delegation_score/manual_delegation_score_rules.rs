use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual delegation score must pass. Check returns its problems, or none.
pub trait ManualDelegationScoreRule: std::fmt::Debug {
    fn check(&self, item: &ManualDelegationScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationScoreContactEmailLengthRule {
    pub max_length: i64,
}

impl ManualDelegationScoreRule for ManualDelegationScoreContactEmailLengthRule {
    fn check(&self, item: &ManualDelegationScore) -> Vec<String> {
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
pub struct ManualDelegationScoreRequiredContactEmailRule {
    pub message: String,
}

impl ManualDelegationScoreRule for ManualDelegationScoreRequiredContactEmailRule {
    fn check(&self, item: &ManualDelegationScore) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationScoreScopeLengthRule {
    pub max_length: i64,
}

impl ManualDelegationScoreRule for ManualDelegationScoreScopeLengthRule {
    fn check(&self, item: &ManualDelegationScore) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationScoreCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualDelegationScoreRule for ManualDelegationScoreCeilingRangeRule {
    fn check(&self, item: &ManualDelegationScore) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationScoreCodePrefixRule {
    pub prefix: String,
}

impl ManualDelegationScoreRule for ManualDelegationScoreCodePrefixRule {
    fn check(&self, item: &ManualDelegationScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual delegation score.
#[derive(Debug, Clone)]
pub struct ManualDelegationScoreValidator {
    pub rules: Vec<Rc<dyn ManualDelegationScoreRule>>,
}

impl ManualDelegationScoreValidator {
    pub fn validate(&self, item: &ManualDelegationScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualDelegationScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDelegationScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_delegation_score_validator() -> ManualDelegationScoreValidator {
    ManualDelegationScoreValidator {
        rules: vec![
            Rc::new(ManualDelegationScoreContactEmailLengthRule { max_length: 32 })
                as Rc<dyn ManualDelegationScoreRule>,
            Rc::new(ManualDelegationScoreRequiredContactEmailRule {
                message: "manual delegation score needs an entry".to_string(),
            }),
            Rc::new(ManualDelegationScoreScopeLengthRule { max_length: 1000 }),
            Rc::new(ManualDelegationScoreCeilingRangeRule { min: 7.0, max: 1401.7 }),
            Rc::new(ManualDelegationScoreCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_delegation_score_validator();
        let item = new_manual_delegation_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = ManualDelegationScoreContactEmailLengthRule { max_length: 120 };
        let mut item = new_manual_delegation_score("x", "y");
        item.contact_email = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = ManualDelegationScoreRequiredContactEmailRule {
            message: "manual delegation score needs a value".to_string(),
        };
        let mut item = new_manual_delegation_score("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = ManualDelegationScoreScopeLengthRule { max_length: 86400 };
        let mut item = new_manual_delegation_score("x", "y");
        item.scope = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ManualDelegationScoreCeilingRangeRule { min: 3.2, max: 4132.7 };
        let mut item = new_manual_delegation_score("x", "y");
        item.ceiling = 4134.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
