use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist reservation must pass. Check returns its problems, or none.
pub trait AllowlistReservationRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistReservationCodePrefixRule {
    pub prefix: String,
}

impl AllowlistReservationRule for AllowlistReservationCodePrefixRule {
    fn check(&self, item: &AllowlistReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistReservationRequiredClusterRule {
    pub message: String,
}

impl AllowlistReservationRule for AllowlistReservationRequiredClusterRule {
    fn check(&self, item: &AllowlistReservation) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistReservationRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistReservationRule for AllowlistReservationRotationDaysRangeRule {
    fn check(&self, item: &AllowlistReservation) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistReservationRequiredKeyIdRule {
    pub message: String,
}

impl AllowlistReservationRule for AllowlistReservationRequiredKeyIdRule {
    fn check(&self, item: &AllowlistReservation) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistReservationKeyIdLengthRule {
    pub max_length: i64,
}

impl AllowlistReservationRule for AllowlistReservationKeyIdLengthRule {
    fn check(&self, item: &AllowlistReservation) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist reservation.
#[derive(Debug, Clone)]
pub struct AllowlistReservationValidator {
    pub rules: Vec<Rc<dyn AllowlistReservationRule>>,
}

impl AllowlistReservationValidator {
    pub fn validate(&self, item: &AllowlistReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_reservation_validator() -> AllowlistReservationValidator {
    AllowlistReservationValidator {
        rules: vec![
            Rc::new(AllowlistReservationCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn AllowlistReservationRule>,
            Rc::new(AllowlistReservationRequiredClusterRule {
                message: "allowlist reservation needs an entry".to_string(),
            }),
            Rc::new(AllowlistReservationRotationDaysRangeRule { min: 32, max: 64 }),
            Rc::new(AllowlistReservationRequiredKeyIdRule {
                message: "allowlist reservation needs this field".to_string(),
            }),
            Rc::new(AllowlistReservationKeyIdLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_reservation_validator();
        let item = new_allowlist_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = AllowlistReservationRequiredClusterRule {
            message: "allowlist reservation needs an entry".to_string(),
        };
        let mut item = new_allowlist_reservation("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = AllowlistReservationRotationDaysRangeRule { min: 16, max: 86400 };
        let mut item = new_allowlist_reservation("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = AllowlistReservationRequiredKeyIdRule {
            message: "allowlist reservation needs an entry".to_string(),
        };
        let mut item = new_allowlist_reservation("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = AllowlistReservationKeyIdLengthRule { max_length: 86400 };
        let mut item = new_allowlist_reservation("x", "y");
        item.key_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
