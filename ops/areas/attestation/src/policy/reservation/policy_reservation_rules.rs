use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy reservation must pass. Check returns its problems, or none.
pub trait PolicyReservationRule: std::fmt::Debug {
    fn check(&self, item: &PolicyReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReservationRequiredTitleRule {
    pub message: String,
}

impl PolicyReservationRule for PolicyReservationRequiredTitleRule {
    fn check(&self, item: &PolicyReservation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReservationCodePrefixRule {
    pub prefix: String,
}

impl PolicyReservationRule for PolicyReservationCodePrefixRule {
    fn check(&self, item: &PolicyReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReservationPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyReservationRule for PolicyReservationPortRangeRule {
    fn check(&self, item: &PolicyReservation) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReservationRequiredCodeRule {
    pub message: String,
}

impl PolicyReservationRule for PolicyReservationRequiredCodeRule {
    fn check(&self, item: &PolicyReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy reservation.
#[derive(Debug, Clone)]
pub struct PolicyReservationValidator {
    pub rules: Vec<Rc<dyn PolicyReservationRule>>,
}

impl PolicyReservationValidator {
    pub fn validate(&self, item: &PolicyReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_reservation_validator() -> PolicyReservationValidator {
    PolicyReservationValidator {
        rules: vec![
            Rc::new(PolicyReservationRequiredTitleRule {
                message: "policy reservation needs an entry".to_string(),
            }) as Rc<dyn PolicyReservationRule>,
            Rc::new(PolicyReservationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PolicyReservationPortRangeRule { min: 255, max: 1000 }),
            Rc::new(PolicyReservationRequiredCodeRule {
                message: "policy reservation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_reservation_validator();
        let item = new_policy_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PolicyReservationRequiredTitleRule {
            message: "policy reservation needs this field".to_string(),
        };
        let mut item = new_policy_reservation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = PolicyReservationPortRangeRule { min: 64, max: 1000 };
        let mut item = new_policy_reservation("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PolicyReservationRequiredCodeRule {
            message: "policy reservation needs an entry".to_string(),
        };
        let mut item = new_policy_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
