use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation policy must pass. Check returns its problems, or none.
pub trait ReservationPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ReservationPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPolicyCodeLengthRule {
    pub max_length: i64,
}

impl ReservationPolicyRule for ReservationPolicyCodeLengthRule {
    fn check(&self, item: &ReservationPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPolicyArchivedGuardRule {
    pub limit: i64,
}

impl ReservationPolicyRule for ReservationPolicyArchivedGuardRule {
    fn check(&self, item: &ReservationPolicy) -> Vec<String> {
        if item.archived && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while archived",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPolicyRequiredCategoryRule {
    pub message: String,
}

impl ReservationPolicyRule for ReservationPolicyRequiredCategoryRule {
    fn check(&self, item: &ReservationPolicy) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPolicyNameLengthRule {
    pub max_length: i64,
}

impl ReservationPolicyRule for ReservationPolicyNameLengthRule {
    fn check(&self, item: &ReservationPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPolicyOwnerIdLengthRule {
    pub max_length: i64,
}

impl ReservationPolicyRule for ReservationPolicyOwnerIdLengthRule {
    fn check(&self, item: &ReservationPolicy) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPolicyPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationPolicyRule for ReservationPolicyPortRangeRule {
    fn check(&self, item: &ReservationPolicy) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation policy.
#[derive(Debug, Clone)]
pub struct ReservationPolicyValidator {
    pub rules: Vec<Rc<dyn ReservationPolicyRule>>,
}

impl ReservationPolicyValidator {
    pub fn validate(&self, item: &ReservationPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_policy_validator() -> ReservationPolicyValidator {
    ReservationPolicyValidator {
        rules: vec![
            Rc::new(ReservationPolicyCodeLengthRule { max_length: 3600 }) as Rc<dyn ReservationPolicyRule>,
            Rc::new(ReservationPolicyArchivedGuardRule { limit: 80 }),
            Rc::new(ReservationPolicyRequiredCategoryRule {
                message: "reservation policy needs an entry".to_string(),
            }),
            Rc::new(ReservationPolicyNameLengthRule { max_length: 120 }),
            Rc::new(ReservationPolicyOwnerIdLengthRule { max_length: 86400 }),
            Rc::new(ReservationPolicyPortRangeRule { min: 32, max: 255 }),
        ],
    }
}
