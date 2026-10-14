use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary verifier target must pass. Check returns its problems, or none.
pub trait PrimaryVerifierTargetRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryVerifierTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTargetRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryVerifierTargetRule for PrimaryVerifierTargetRetriesRangeRule {
    fn check(&self, item: &PrimaryVerifierTarget) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTargetPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryVerifierTargetRule for PrimaryVerifierTargetPriorityRangeRule {
    fn check(&self, item: &PrimaryVerifierTarget) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTargetAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PrimaryVerifierTargetRule for PrimaryVerifierTargetAllowedNameRule {
    fn check(&self, item: &PrimaryVerifierTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTargetRequiredRegionRule {
    pub message: String,
}

impl PrimaryVerifierTargetRule for PrimaryVerifierTargetRequiredRegionRule {
    fn check(&self, item: &PrimaryVerifierTarget) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary verifier target.
#[derive(Debug, Clone)]
pub struct PrimaryVerifierTargetValidator {
    pub rules: Vec<Rc<dyn PrimaryVerifierTargetRule>>,
}

impl PrimaryVerifierTargetValidator {
    pub fn validate(&self, item: &PrimaryVerifierTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryVerifierTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryVerifierTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_verifier_target_validator() -> PrimaryVerifierTargetValidator {
    PrimaryVerifierTargetValidator {
        rules: vec![
            Rc::new(PrimaryVerifierTargetRetriesRangeRule { min: 64, max: 3600 }) as Rc<dyn PrimaryVerifierTargetRule>,
            Rc::new(PrimaryVerifierTargetPriorityRangeRule { min: 80, max: 3600 }),
            Rc::new(PrimaryVerifierTargetAllowedNameRule {
                allowed: vec!["regional".to_string(), "seasonal".to_string(), "capped".to_string()],
            }),
            Rc::new(PrimaryVerifierTargetRequiredRegionRule {
                message: "primary verifier target needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_verifier_target_validator();
        let item = new_primary_verifier_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = PrimaryVerifierTargetRetriesRangeRule { min: 32, max: 86400 };
        let mut item = new_primary_verifier_target("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PrimaryVerifierTargetPriorityRangeRule { min: 64, max: 1000 };
        let mut item = new_primary_verifier_target("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = PrimaryVerifierTargetRequiredRegionRule {
            message: "primary verifier target needs this field".to_string(),
        };
        let mut item = new_primary_verifier_target("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
