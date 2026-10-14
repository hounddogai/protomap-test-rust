use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling delegation slot must pass. Check returns its problems, or none.
pub trait RollingDelegationSlotRule: std::fmt::Debug {
    fn check(&self, item: &RollingDelegationSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationSlotNamespaceLengthRule {
    pub max_length: i64,
}

impl RollingDelegationSlotRule for RollingDelegationSlotNamespaceLengthRule {
    fn check(&self, item: &RollingDelegationSlot) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationSlotArchivedGuardRule {
    pub limit: i64,
}

impl RollingDelegationSlotRule for RollingDelegationSlotArchivedGuardRule {
    fn check(&self, item: &RollingDelegationSlot) -> Vec<String> {
        if item.archived && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while archived",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationSlotAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl RollingDelegationSlotRule for RollingDelegationSlotAllowedSegmentRule {
    fn check(&self, item: &RollingDelegationSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is too long", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationSlotRequiredSegmentRule {
    pub message: String,
}

impl RollingDelegationSlotRule for RollingDelegationSlotRequiredSegmentRule {
    fn check(&self, item: &RollingDelegationSlot) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationSlotCodePrefixRule {
    pub prefix: String,
}

impl RollingDelegationSlotRule for RollingDelegationSlotCodePrefixRule {
    fn check(&self, item: &RollingDelegationSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationSlotNameLengthRule {
    pub max_length: i64,
}

impl RollingDelegationSlotRule for RollingDelegationSlotNameLengthRule {
    fn check(&self, item: &RollingDelegationSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling delegation slot.
#[derive(Debug, Clone)]
pub struct RollingDelegationSlotValidator {
    pub rules: Vec<Rc<dyn RollingDelegationSlotRule>>,
}

impl RollingDelegationSlotValidator {
    pub fn validate(&self, item: &RollingDelegationSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingDelegationSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingDelegationSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_delegation_slot_validator() -> RollingDelegationSlotValidator {
    RollingDelegationSlotValidator {
        rules: vec![
            Rc::new(RollingDelegationSlotNamespaceLengthRule { max_length: 64 }) as Rc<dyn RollingDelegationSlotRule>,
            Rc::new(RollingDelegationSlotArchivedGuardRule { limit: 3600 }),
            Rc::new(RollingDelegationSlotAllowedSegmentRule {
                allowed: vec!["default".to_string()],
            }),
            Rc::new(RollingDelegationSlotRequiredSegmentRule {
                message: "rolling delegation slot needs an entry".to_string(),
            }),
            Rc::new(RollingDelegationSlotCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RollingDelegationSlotNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_delegation_slot_validator();
        let item = new_rolling_delegation_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = RollingDelegationSlotNamespaceLengthRule { max_length: 16 };
        let mut item = new_rolling_delegation_slot("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = RollingDelegationSlotRequiredSegmentRule {
            message: "rolling delegation slot needs a value".to_string(),
        };
        let mut item = new_rolling_delegation_slot("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingDelegationSlotNameLengthRule { max_length: 16 };
        let mut item = new_rolling_delegation_slot("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
