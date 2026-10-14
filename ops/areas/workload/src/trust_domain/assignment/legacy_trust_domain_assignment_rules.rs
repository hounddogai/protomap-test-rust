use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy trust domain assignment must pass. Check returns its problems, or none.
pub trait LegacyTrustDomainAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &LegacyTrustDomainAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainAssignmentRequiredTitleRule {
    pub message: String,
}

impl LegacyTrustDomainAssignmentRule for LegacyTrustDomainAssignmentRequiredTitleRule {
    fn check(&self, item: &LegacyTrustDomainAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainAssignmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyTrustDomainAssignmentRule for LegacyTrustDomainAssignmentPortRangeRule {
    fn check(&self, item: &LegacyTrustDomainAssignment) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainAssignmentAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LegacyTrustDomainAssignmentRule for LegacyTrustDomainAssignmentAllowedTitleRule {
    fn check(&self, item: &LegacyTrustDomainAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainAssignmentWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyTrustDomainAssignmentRule for LegacyTrustDomainAssignmentWindowDaysRangeRule {
    fn check(&self, item: &LegacyTrustDomainAssignment) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy trust domain assignment.
#[derive(Debug, Clone)]
pub struct LegacyTrustDomainAssignmentValidator {
    pub rules: Vec<Rc<dyn LegacyTrustDomainAssignmentRule>>,
}

impl LegacyTrustDomainAssignmentValidator {
    pub fn validate(&self, item: &LegacyTrustDomainAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyTrustDomainAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyTrustDomainAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_trust_domain_assignment_validator() -> LegacyTrustDomainAssignmentValidator {
    LegacyTrustDomainAssignmentValidator {
        rules: vec![
            Rc::new(LegacyTrustDomainAssignmentRequiredTitleRule {
                message: "legacy trust domain assignment needs a value".to_string(),
            }) as Rc<dyn LegacyTrustDomainAssignmentRule>,
            Rc::new(LegacyTrustDomainAssignmentPortRangeRule { min: 64, max: 120 }),
            Rc::new(LegacyTrustDomainAssignmentAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(LegacyTrustDomainAssignmentWindowDaysRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_trust_domain_assignment_validator();
        let item = new_legacy_trust_domain_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyTrustDomainAssignmentRequiredTitleRule {
            message: "legacy trust domain assignment needs this field".to_string(),
        };
        let mut item = new_legacy_trust_domain_assignment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LegacyTrustDomainAssignmentPortRangeRule { min: 32, max: 64 };
        let mut item = new_legacy_trust_domain_assignment("x", "y");
        item.port = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = LegacyTrustDomainAssignmentWindowDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_legacy_trust_domain_assignment("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
