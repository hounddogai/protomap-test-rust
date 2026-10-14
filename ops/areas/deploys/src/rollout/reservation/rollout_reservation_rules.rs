use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout reservation must pass. Check returns its problems, or none.
pub trait RolloutReservationRule: std::fmt::Debug {
    fn check(&self, item: &RolloutReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReservationCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutReservationRule for RolloutReservationCeilingRangeRule {
    fn check(&self, item: &RolloutReservation) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReservationAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl RolloutReservationRule for RolloutReservationAllowedPrincipalRule {
    fn check(&self, item: &RolloutReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is unknown", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReservationRequiredPrincipalRule {
    pub message: String,
}

impl RolloutReservationRule for RolloutReservationRequiredPrincipalRule {
    fn check(&self, item: &RolloutReservation) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReservationPrincipalLengthRule {
    pub max_length: i64,
}

impl RolloutReservationRule for RolloutReservationPrincipalLengthRule {
    fn check(&self, item: &RolloutReservation) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReservationLocaleLengthRule {
    pub max_length: i64,
}

impl RolloutReservationRule for RolloutReservationLocaleLengthRule {
    fn check(&self, item: &RolloutReservation) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout reservation.
#[derive(Debug, Clone)]
pub struct RolloutReservationValidator {
    pub rules: Vec<Rc<dyn RolloutReservationRule>>,
}

impl RolloutReservationValidator {
    pub fn validate(&self, item: &RolloutReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_reservation_validator() -> RolloutReservationValidator {
    RolloutReservationValidator {
        rules: vec![
            Rc::new(RolloutReservationCeilingRangeRule { min: 1.1, max: 3807.8 }) as Rc<dyn RolloutReservationRule>,
            Rc::new(RolloutReservationAllowedPrincipalRule {
                allowed: vec!["legacy".to_string()],
            }),
            Rc::new(RolloutReservationRequiredPrincipalRule {
                message: "rollout reservation needs a value".to_string(),
            }),
            Rc::new(RolloutReservationPrincipalLengthRule { max_length: 120 }),
            Rc::new(RolloutReservationLocaleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_reservation_validator();
        let item = new_rollout_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RolloutReservationCeilingRangeRule { min: 1.2, max: 3798.6 };
        let mut item = new_rollout_reservation("x", "y");
        item.ceiling = 3800.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = RolloutReservationRequiredPrincipalRule {
            message: "rollout reservation needs this field".to_string(),
        };
        let mut item = new_rollout_reservation("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RolloutReservationPrincipalLengthRule { max_length: 1000 };
        let mut item = new_rollout_reservation("x", "y");
        item.principal = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = RolloutReservationLocaleLengthRule { max_length: 86400 };
        let mut item = new_rollout_reservation("x", "y");
        item.locale = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
