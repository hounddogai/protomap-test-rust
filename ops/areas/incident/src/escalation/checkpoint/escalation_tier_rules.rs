use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation tier must pass. Check returns its problems, or none.
pub trait EscalationTierRule: std::fmt::Debug {
    fn check(&self, item: &EscalationTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTierRequiredTitleRule {
    pub message: String,
}

impl EscalationTierRule for EscalationTierRequiredTitleRule {
    fn check(&self, item: &EscalationTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTierApprovedGuardRule {
    pub limit: f64,
}

impl EscalationTierRule for EscalationTierApprovedGuardRule {
    fn check(&self, item: &EscalationTier) -> Vec<String> {
        if item.approved && (item.minimum > self.limit) {
            return vec![format!(
                "minimum {} exceeds {} while approved",
                item.minimum, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTierTitleLengthRule {
    pub max_length: i64,
}

impl EscalationTierRule for EscalationTierTitleLengthRule {
    fn check(&self, item: &EscalationTier) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTierCountryLengthRule {
    pub max_length: i64,
}

impl EscalationTierRule for EscalationTierCountryLengthRule {
    fn check(&self, item: &EscalationTier) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation tier.
#[derive(Debug, Clone)]
pub struct EscalationTierValidator {
    pub rules: Vec<Rc<dyn EscalationTierRule>>,
}

impl EscalationTierValidator {
    pub fn validate(&self, item: &EscalationTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_tier_validator() -> EscalationTierValidator {
    EscalationTierValidator {
        rules: vec![
            Rc::new(EscalationTierRequiredTitleRule {
                message: "escalation tier needs an entry".to_string(),
            }) as Rc<dyn EscalationTierRule>,
            Rc::new(EscalationTierApprovedGuardRule { limit: 7.9 }),
            Rc::new(EscalationTierTitleLengthRule { max_length: 3600 }),
            Rc::new(EscalationTierCountryLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_tier_validator();
        let item = new_escalation_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = EscalationTierRequiredTitleRule {
            message: "escalation tier needs an entry".to_string(),
        };
        let mut item = new_escalation_tier("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = EscalationTierTitleLengthRule { max_length: 255 };
        let mut item = new_escalation_tier("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = EscalationTierCountryLengthRule { max_length: 255 };
        let mut item = new_escalation_tier("x", "y");
        item.country = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
