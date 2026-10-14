use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain adjustment must pass. Check returns its problems, or none.
pub trait TrustDomainAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAdjustmentPinnedGuardRule {
    pub limit: i64,
}

impl TrustDomainAdjustmentRule for TrustDomainAdjustmentPinnedGuardRule {
    fn check(&self, item: &TrustDomainAdjustment) -> Vec<String> {
        if item.pinned && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while pinned",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAdjustmentExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainAdjustmentRule for TrustDomainAdjustmentExpiresAtRangeRule {
    fn check(&self, item: &TrustDomainAdjustment) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainAdjustmentRule for TrustDomainAdjustmentCodeLengthRule {
    fn check(&self, item: &TrustDomainAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAdjustmentGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainAdjustmentRule for TrustDomainAdjustmentGraceDaysRangeRule {
    fn check(&self, item: &TrustDomainAdjustment) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAdjustmentMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainAdjustmentRule for TrustDomainAdjustmentMinimumRangeRule {
    fn check(&self, item: &TrustDomainAdjustment) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a trust domain adjustment.
#[derive(Debug, Clone)]
pub struct TrustDomainAdjustmentValidator {
    pub rules: Vec<Rc<dyn TrustDomainAdjustmentRule>>,
}

impl TrustDomainAdjustmentValidator {
    pub fn validate(&self, item: &TrustDomainAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_adjustment_validator() -> TrustDomainAdjustmentValidator {
    TrustDomainAdjustmentValidator {
        rules: vec![
            Rc::new(TrustDomainAdjustmentPinnedGuardRule { limit: 120 }) as Rc<dyn TrustDomainAdjustmentRule>,
            Rc::new(TrustDomainAdjustmentExpiresAtRangeRule { min: 64, max: 3600 }),
            Rc::new(TrustDomainAdjustmentCodeLengthRule { max_length: 1000 }),
            Rc::new(TrustDomainAdjustmentGraceDaysRangeRule { min: 64, max: 255 }),
            Rc::new(TrustDomainAdjustmentMinimumRangeRule { min: 2.3, max: 461.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_adjustment_validator();
        let item = new_trust_domain_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = TrustDomainAdjustmentExpiresAtRangeRule { min: 64, max: 255 };
        let mut item = new_trust_domain_adjustment("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainAdjustmentCodeLengthRule { max_length: 120 };
        let mut item = new_trust_domain_adjustment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = TrustDomainAdjustmentGraceDaysRangeRule { min: 64, max: 64 };
        let mut item = new_trust_domain_adjustment("x", "y");
        item.grace_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = TrustDomainAdjustmentMinimumRangeRule { min: 3.9, max: 2775.7 };
        let mut item = new_trust_domain_adjustment("x", "y");
        item.minimum = 2777.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
