use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped overage reservation must pass. Check returns its problems, or none.
pub trait CappedOverageReservationRule: std::fmt::Debug {
    fn check(&self, item: &CappedOverageReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageReservationRequiredCodeRule {
    pub message: String,
}

impl CappedOverageReservationRule for CappedOverageReservationRequiredCodeRule {
    fn check(&self, item: &CappedOverageReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageReservationClusterLengthRule {
    pub max_length: i64,
}

impl CappedOverageReservationRule for CappedOverageReservationClusterLengthRule {
    fn check(&self, item: &CappedOverageReservation) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageReservationTitleLengthRule {
    pub max_length: i64,
}

impl CappedOverageReservationRule for CappedOverageReservationTitleLengthRule {
    fn check(&self, item: &CappedOverageReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageReservationCodeLengthRule {
    pub max_length: i64,
}

impl CappedOverageReservationRule for CappedOverageReservationCodeLengthRule {
    fn check(&self, item: &CappedOverageReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageReservationCurrencyLengthRule {
    pub max_length: i64,
}

impl CappedOverageReservationRule for CappedOverageReservationCurrencyLengthRule {
    fn check(&self, item: &CappedOverageReservation) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped overage reservation.
#[derive(Debug, Clone)]
pub struct CappedOverageReservationValidator {
    pub rules: Vec<Rc<dyn CappedOverageReservationRule>>,
}

impl CappedOverageReservationValidator {
    pub fn validate(&self, item: &CappedOverageReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedOverageReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedOverageReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_overage_reservation_validator() -> CappedOverageReservationValidator {
    CappedOverageReservationValidator {
        rules: vec![
            Rc::new(CappedOverageReservationRequiredCodeRule {
                message: "capped overage reservation needs a value".to_string(),
            }) as Rc<dyn CappedOverageReservationRule>,
            Rc::new(CappedOverageReservationClusterLengthRule { max_length: 80 }),
            Rc::new(CappedOverageReservationTitleLengthRule { max_length: 64 }),
            Rc::new(CappedOverageReservationCodeLengthRule { max_length: 32 }),
            Rc::new(CappedOverageReservationCurrencyLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_overage_reservation_validator();
        let item = new_capped_overage_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedOverageReservationRequiredCodeRule {
            message: "capped overage reservation needs a value".to_string(),
        };
        let mut item = new_capped_overage_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = CappedOverageReservationClusterLengthRule { max_length: 3600 };
        let mut item = new_capped_overage_reservation("x", "y");
        item.cluster = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CappedOverageReservationTitleLengthRule { max_length: 1000 };
        let mut item = new_capped_overage_reservation("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedOverageReservationCodeLengthRule { max_length: 1000 };
        let mut item = new_capped_overage_reservation("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = CappedOverageReservationCurrencyLengthRule { max_length: 80 };
        let mut item = new_capped_overage_reservation("x", "y");
        item.currency = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
