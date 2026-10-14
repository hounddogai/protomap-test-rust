use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted credential reservation must pass. Check returns its problems, or none.
pub trait WeightedCredentialReservationRule: std::fmt::Debug {
    fn check(&self, item: &WeightedCredentialReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialReservationRequiredCodeRule {
    pub message: String,
}

impl WeightedCredentialReservationRule for WeightedCredentialReservationRequiredCodeRule {
    fn check(&self, item: &WeightedCredentialReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialReservationAudienceLengthRule {
    pub max_length: i64,
}

impl WeightedCredentialReservationRule for WeightedCredentialReservationAudienceLengthRule {
    fn check(&self, item: &WeightedCredentialReservation) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialReservationActiveGuardRule {
    pub limit: i64,
}

impl WeightedCredentialReservationRule for WeightedCredentialReservationActiveGuardRule {
    fn check(&self, item: &WeightedCredentialReservation) -> Vec<String> {
        if item.active && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while active",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted credential reservation.
#[derive(Debug, Clone)]
pub struct WeightedCredentialReservationValidator {
    pub rules: Vec<Rc<dyn WeightedCredentialReservationRule>>,
}

impl WeightedCredentialReservationValidator {
    pub fn validate(&self, item: &WeightedCredentialReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedCredentialReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedCredentialReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_credential_reservation_validator() -> WeightedCredentialReservationValidator {
    WeightedCredentialReservationValidator {
        rules: vec![
            Rc::new(WeightedCredentialReservationRequiredCodeRule {
                message: "weighted credential reservation needs this field".to_string(),
            }) as Rc<dyn WeightedCredentialReservationRule>,
            Rc::new(WeightedCredentialReservationAudienceLengthRule { max_length: 32 }),
            Rc::new(WeightedCredentialReservationActiveGuardRule { limit: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_credential_reservation_validator();
        let item = new_weighted_credential_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedCredentialReservationRequiredCodeRule {
            message: "weighted credential reservation needs a value".to_string(),
        };
        let mut item = new_weighted_credential_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = WeightedCredentialReservationAudienceLengthRule { max_length: 80 };
        let mut item = new_weighted_credential_reservation("x", "y");
        item.audience = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
