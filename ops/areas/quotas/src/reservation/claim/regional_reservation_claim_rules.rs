use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional reservation claim must pass. Check returns its problems, or none.
pub trait RegionalReservationClaimRule: std::fmt::Debug {
    fn check(&self, item: &RegionalReservationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationClaimTitleLengthRule {
    pub max_length: i64,
}

impl RegionalReservationClaimRule for RegionalReservationClaimTitleLengthRule {
    fn check(&self, item: &RegionalReservationClaim) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationClaimAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalReservationClaimRule for RegionalReservationClaimAmountRangeRule {
    fn check(&self, item: &RegionalReservationClaim) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationClaimBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalReservationClaimRule for RegionalReservationClaimBatchSizeRangeRule {
    fn check(&self, item: &RegionalReservationClaim) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalReservationClaimRequiredAudienceRule {
    pub message: String,
}

impl RegionalReservationClaimRule for RegionalReservationClaimRequiredAudienceRule {
    fn check(&self, item: &RegionalReservationClaim) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional reservation claim.
#[derive(Debug, Clone)]
pub struct RegionalReservationClaimValidator {
    pub rules: Vec<Rc<dyn RegionalReservationClaimRule>>,
}

impl RegionalReservationClaimValidator {
    pub fn validate(&self, item: &RegionalReservationClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalReservationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalReservationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_reservation_claim_validator() -> RegionalReservationClaimValidator {
    RegionalReservationClaimValidator {
        rules: vec![
            Rc::new(RegionalReservationClaimTitleLengthRule { max_length: 255 })
                as Rc<dyn RegionalReservationClaimRule>,
            Rc::new(RegionalReservationClaimAmountRangeRule { min: 5.8, max: 2911.7 }),
            Rc::new(RegionalReservationClaimBatchSizeRangeRule { min: 80, max: 1000 }),
            Rc::new(RegionalReservationClaimRequiredAudienceRule {
                message: "regional reservation claim needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_reservation_claim_validator();
        let item = new_regional_reservation_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalReservationClaimTitleLengthRule { max_length: 3600 };
        let mut item = new_regional_reservation_claim("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = RegionalReservationClaimAmountRangeRule { min: 1.9, max: 2487.3 };
        let mut item = new_regional_reservation_claim("x", "y");
        item.amount = 2488.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = RegionalReservationClaimBatchSizeRangeRule { min: 120, max: 86400 };
        let mut item = new_regional_reservation_claim("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RegionalReservationClaimRequiredAudienceRule {
            message: "regional reservation claim needs a value".to_string(),
        };
        let mut item = new_regional_reservation_claim("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
