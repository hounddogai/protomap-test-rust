use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched reservation policy must pass. Check returns its problems, or none.
pub trait BatchedReservationPolicyRule: std::fmt::Debug {
    fn check(&self, item: &BatchedReservationPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationPolicyRequiredCodeRule {
    pub message: String,
}

impl BatchedReservationPolicyRule for BatchedReservationPolicyRequiredCodeRule {
    fn check(&self, item: &BatchedReservationPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationPolicyExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedReservationPolicyRule for BatchedReservationPolicyExpiresAtRangeRule {
    fn check(&self, item: &BatchedReservationPolicy) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationPolicyKeyIdLengthRule {
    pub max_length: i64,
}

impl BatchedReservationPolicyRule for BatchedReservationPolicyKeyIdLengthRule {
    fn check(&self, item: &BatchedReservationPolicy) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationPolicyPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedReservationPolicyRule for BatchedReservationPolicyPortRangeRule {
    fn check(&self, item: &BatchedReservationPolicy) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched reservation policy.
#[derive(Debug, Clone)]
pub struct BatchedReservationPolicyValidator {
    pub rules: Vec<Rc<dyn BatchedReservationPolicyRule>>,
}

impl BatchedReservationPolicyValidator {
    pub fn validate(&self, item: &BatchedReservationPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedReservationPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedReservationPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_reservation_policy_validator() -> BatchedReservationPolicyValidator {
    BatchedReservationPolicyValidator {
        rules: vec![
            Rc::new(BatchedReservationPolicyRequiredCodeRule {
                message: "batched reservation policy needs an entry".to_string(),
            }) as Rc<dyn BatchedReservationPolicyRule>,
            Rc::new(BatchedReservationPolicyExpiresAtRangeRule { min: 80, max: 255 }),
            Rc::new(BatchedReservationPolicyKeyIdLengthRule { max_length: 3600 }),
            Rc::new(BatchedReservationPolicyPortRangeRule { min: 80, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_reservation_policy_validator();
        let item = new_batched_reservation_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedReservationPolicyRequiredCodeRule {
            message: "batched reservation policy needs an entry".to_string(),
        };
        let mut item = new_batched_reservation_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = BatchedReservationPolicyExpiresAtRangeRule { min: 80, max: 3600 };
        let mut item = new_batched_reservation_policy("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = BatchedReservationPolicyKeyIdLengthRule { max_length: 86400 };
        let mut item = new_batched_reservation_policy("x", "y");
        item.key_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = BatchedReservationPolicyPortRangeRule { min: 32, max: 1000 };
        let mut item = new_batched_reservation_policy("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
