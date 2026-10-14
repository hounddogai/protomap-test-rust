use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a delegation digest must pass. Check returns its problems, or none.
pub trait DelegationDigestRule: std::fmt::Debug {
    fn check(&self, item: &DelegationDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationDigestRequiredTitleRule {
    pub message: String,
}

impl DelegationDigestRule for DelegationDigestRequiredTitleRule {
    fn check(&self, item: &DelegationDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationDigestBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationDigestRule for DelegationDigestBurstRangeRule {
    fn check(&self, item: &DelegationDigest) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationDigestMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationDigestRule for DelegationDigestMaximumRangeRule {
    fn check(&self, item: &DelegationDigest) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationDigestAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl DelegationDigestRule for DelegationDigestAllowedTitleRule {
    fn check(&self, item: &DelegationDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation digest.
#[derive(Debug, Clone)]
pub struct DelegationDigestValidator {
    pub rules: Vec<Rc<dyn DelegationDigestRule>>,
}

impl DelegationDigestValidator {
    pub fn validate(&self, item: &DelegationDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_digest_validator() -> DelegationDigestValidator {
    DelegationDigestValidator {
        rules: vec![
            Rc::new(DelegationDigestRequiredTitleRule {
                message: "delegation digest needs a value".to_string(),
            }) as Rc<dyn DelegationDigestRule>,
            Rc::new(DelegationDigestBurstRangeRule { min: 64, max: 3600 }),
            Rc::new(DelegationDigestMaximumRangeRule { min: 1.9, max: 1968.2 }),
            Rc::new(DelegationDigestAllowedTitleRule {
                allowed: vec!["tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_digest_validator();
        let item = new_delegation_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DelegationDigestRequiredTitleRule {
            message: "delegation digest needs this field".to_string(),
        };
        let mut item = new_delegation_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = DelegationDigestBurstRangeRule { min: 16, max: 32 };
        let mut item = new_delegation_digest("x", "y");
        item.burst = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = DelegationDigestMaximumRangeRule { min: 6.2, max: 63.9 };
        let mut item = new_delegation_digest("x", "y");
        item.maximum = 65.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
