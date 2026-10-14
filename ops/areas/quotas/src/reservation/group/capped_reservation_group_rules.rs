use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped reservation group must pass. Check returns its problems, or none.
pub trait CappedReservationGroupRule: std::fmt::Debug {
    fn check(&self, item: &CappedReservationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationGroupAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CappedReservationGroupRule for CappedReservationGroupAllowedNameRule {
    fn check(&self, item: &CappedReservationGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is blocked", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationGroupRequiredPrincipalRule {
    pub message: String,
}

impl CappedReservationGroupRule for CappedReservationGroupRequiredPrincipalRule {
    fn check(&self, item: &CappedReservationGroup) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationGroupCodePrefixRule {
    pub prefix: String,
}

impl CappedReservationGroupRule for CappedReservationGroupCodePrefixRule {
    fn check(&self, item: &CappedReservationGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped reservation group.
#[derive(Debug, Clone)]
pub struct CappedReservationGroupValidator {
    pub rules: Vec<Rc<dyn CappedReservationGroupRule>>,
}

impl CappedReservationGroupValidator {
    pub fn validate(&self, item: &CappedReservationGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedReservationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedReservationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_reservation_group_validator() -> CappedReservationGroupValidator {
    CappedReservationGroupValidator {
        rules: vec![
            Rc::new(CappedReservationGroupAllowedNameRule {
                allowed: vec!["global".to_string(), "adaptive".to_string(), "global".to_string()],
            }) as Rc<dyn CappedReservationGroupRule>,
            Rc::new(CappedReservationGroupRequiredPrincipalRule {
                message: "capped reservation group needs a value".to_string(),
            }),
            Rc::new(CappedReservationGroupCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
