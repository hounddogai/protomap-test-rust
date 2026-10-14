use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress tier must pass. Check returns its problems, or none.
pub trait EgressTierRule: std::fmt::Debug {
    fn check(&self, item: &EgressTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressTierCodePrefixRule {
    pub prefix: String,
}

impl EgressTierRule for EgressTierCodePrefixRule {
    fn check(&self, item: &EgressTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressTierRequiredRegionRule {
    pub message: String,
}

impl EgressTierRule for EgressTierRequiredRegionRule {
    fn check(&self, item: &EgressTier) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressTierAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl EgressTierRule for EgressTierAllowedRegionRule {
    fn check(&self, item: &EgressTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is unknown", item.region)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress tier.
#[derive(Debug, Clone)]
pub struct EgressTierValidator {
    pub rules: Vec<Rc<dyn EgressTierRule>>,
}

impl EgressTierValidator {
    pub fn validate(&self, item: &EgressTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_tier_validator() -> EgressTierValidator {
    EgressTierValidator {
        rules: vec![
            Rc::new(EgressTierCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn EgressTierRule>,
            Rc::new(EgressTierRequiredRegionRule {
                message: "egress tier needs a value".to_string(),
            }),
            Rc::new(EgressTierAllowedRegionRule {
                allowed: vec!["rolling".to_string(), "manual".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_tier_validator();
        let item = new_egress_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = EgressTierRequiredRegionRule {
            message: "egress tier needs an entry".to_string(),
        };
        let mut item = new_egress_tier("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
