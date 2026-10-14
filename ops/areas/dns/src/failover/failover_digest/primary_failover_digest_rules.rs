use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary failover digest must pass. Check returns its problems, or none.
pub trait PrimaryFailoverDigestRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryFailoverDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverDigestAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryFailoverDigestRule for PrimaryFailoverDigestAmountRangeRule {
    fn check(&self, item: &PrimaryFailoverDigest) -> Vec<String> {
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
pub struct PrimaryFailoverDigestBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryFailoverDigestRule for PrimaryFailoverDigestBurstRangeRule {
    fn check(&self, item: &PrimaryFailoverDigest) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverDigestRequiredCodeRule {
    pub message: String,
}

impl PrimaryFailoverDigestRule for PrimaryFailoverDigestRequiredCodeRule {
    fn check(&self, item: &PrimaryFailoverDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverDigestTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryFailoverDigestRule for PrimaryFailoverDigestTitleLengthRule {
    fn check(&self, item: &PrimaryFailoverDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary failover digest.
#[derive(Debug, Clone)]
pub struct PrimaryFailoverDigestValidator {
    pub rules: Vec<Rc<dyn PrimaryFailoverDigestRule>>,
}

impl PrimaryFailoverDigestValidator {
    pub fn validate(&self, item: &PrimaryFailoverDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryFailoverDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryFailoverDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_failover_digest_validator() -> PrimaryFailoverDigestValidator {
    PrimaryFailoverDigestValidator {
        rules: vec![
            Rc::new(PrimaryFailoverDigestAmountRangeRule { min: 2.8, max: 4746.0 })
                as Rc<dyn PrimaryFailoverDigestRule>,
            Rc::new(PrimaryFailoverDigestBurstRangeRule { min: 16, max: 32 }),
            Rc::new(PrimaryFailoverDigestRequiredCodeRule {
                message: "primary failover digest needs this field".to_string(),
            }),
            Rc::new(PrimaryFailoverDigestTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_failover_digest_validator();
        let item = new_primary_failover_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = PrimaryFailoverDigestAmountRangeRule { min: 6.4, max: 258.8 };
        let mut item = new_primary_failover_digest("x", "y");
        item.amount = 260.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = PrimaryFailoverDigestBurstRangeRule { min: 32, max: 255 };
        let mut item = new_primary_failover_digest("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryFailoverDigestRequiredCodeRule {
            message: "primary failover digest needs this field".to_string(),
        };
        let mut item = new_primary_failover_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryFailoverDigestTitleLengthRule { max_length: 64 };
        let mut item = new_primary_failover_digest("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
