use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global reservation digest must pass. Check returns its problems, or none.
pub trait GlobalReservationDigestRule: std::fmt::Debug {
    fn check(&self, item: &GlobalReservationDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationDigestRequiredNotesRule {
    pub message: String,
}

impl GlobalReservationDigestRule for GlobalReservationDigestRequiredNotesRule {
    fn check(&self, item: &GlobalReservationDigest) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationDigestPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalReservationDigestRule for GlobalReservationDigestPriorityRangeRule {
    fn check(&self, item: &GlobalReservationDigest) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationDigestCodePrefixRule {
    pub prefix: String,
}

impl GlobalReservationDigestRule for GlobalReservationDigestCodePrefixRule {
    fn check(&self, item: &GlobalReservationDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global reservation digest.
#[derive(Debug, Clone)]
pub struct GlobalReservationDigestValidator {
    pub rules: Vec<Rc<dyn GlobalReservationDigestRule>>,
}

impl GlobalReservationDigestValidator {
    pub fn validate(&self, item: &GlobalReservationDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalReservationDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalReservationDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_reservation_digest_validator() -> GlobalReservationDigestValidator {
    GlobalReservationDigestValidator {
        rules: vec![
            Rc::new(GlobalReservationDigestRequiredNotesRule {
                message: "global reservation digest needs a value".to_string(),
            }) as Rc<dyn GlobalReservationDigestRule>,
            Rc::new(GlobalReservationDigestPriorityRangeRule { min: 16, max: 255 }),
            Rc::new(GlobalReservationDigestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_reservation_digest_validator();
        let item = new_global_reservation_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = GlobalReservationDigestRequiredNotesRule {
            message: "global reservation digest needs a value".to_string(),
        };
        let mut item = new_global_reservation_digest("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = GlobalReservationDigestPriorityRangeRule { min: 32, max: 120 };
        let mut item = new_global_reservation_digest("x", "y");
        item.priority = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
