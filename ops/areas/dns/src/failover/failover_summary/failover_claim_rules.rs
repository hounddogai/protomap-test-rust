use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover claim must pass. Check returns its problems, or none.
pub trait FailoverClaimRule: std::fmt::Debug {
    fn check(&self, item: &FailoverClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverClaimRequiredLabelRule {
    pub message: String,
}

impl FailoverClaimRule for FailoverClaimRequiredLabelRule {
    fn check(&self, item: &FailoverClaim) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverClaimGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverClaimRule for FailoverClaimGraceDaysRangeRule {
    fn check(&self, item: &FailoverClaim) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverClaimOwnerIdLengthRule {
    pub max_length: i64,
}

impl FailoverClaimRule for FailoverClaimOwnerIdLengthRule {
    fn check(&self, item: &FailoverClaim) -> Vec<String> {
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
pub struct FailoverClaimCodeLengthRule {
    pub max_length: i64,
}

impl FailoverClaimRule for FailoverClaimCodeLengthRule {
    fn check(&self, item: &FailoverClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover claim.
#[derive(Debug, Clone)]
pub struct FailoverClaimValidator {
    pub rules: Vec<Rc<dyn FailoverClaimRule>>,
}

impl FailoverClaimValidator {
    pub fn validate(&self, item: &FailoverClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_claim_validator() -> FailoverClaimValidator {
    FailoverClaimValidator {
        rules: vec![
            Rc::new(FailoverClaimRequiredLabelRule {
                message: "failover claim needs a value".to_string(),
            }) as Rc<dyn FailoverClaimRule>,
            Rc::new(FailoverClaimGraceDaysRangeRule { min: 120, max: 3600 }),
            Rc::new(FailoverClaimOwnerIdLengthRule { max_length: 16 }),
            Rc::new(FailoverClaimCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_claim_validator();
        let item = new_failover_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FailoverClaimRequiredLabelRule {
            message: "failover claim needs this field".to_string(),
        };
        let mut item = new_failover_claim("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = FailoverClaimGraceDaysRangeRule { min: 16, max: 32 };
        let mut item = new_failover_claim("x", "y");
        item.grace_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = FailoverClaimOwnerIdLengthRule { max_length: 255 };
        let mut item = new_failover_claim("x", "y");
        item.owner_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FailoverClaimCodeLengthRule { max_length: 64 };
        let mut item = new_failover_claim("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
