use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive identity baseline must pass. Check returns its problems, or none.
pub trait AdaptiveIdentityBaselineRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveIdentityBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityBaselinePinnedGuardRule {
    pub limit: i64,
}

impl AdaptiveIdentityBaselineRule for AdaptiveIdentityBaselinePinnedGuardRule {
    fn check(&self, item: &AdaptiveIdentityBaseline) -> Vec<String> {
        if item.pinned && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while pinned",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityBaselineNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveIdentityBaselineRule for AdaptiveIdentityBaselineNameLengthRule {
    fn check(&self, item: &AdaptiveIdentityBaseline) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityBaselineGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveIdentityBaselineRule for AdaptiveIdentityBaselineGraceDaysRangeRule {
    fn check(&self, item: &AdaptiveIdentityBaseline) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityBaselineWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveIdentityBaselineRule for AdaptiveIdentityBaselineWeightRangeRule {
    fn check(&self, item: &AdaptiveIdentityBaseline) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityBaselineCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveIdentityBaselineRule for AdaptiveIdentityBaselineCodeLengthRule {
    fn check(&self, item: &AdaptiveIdentityBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive identity baseline.
#[derive(Debug, Clone)]
pub struct AdaptiveIdentityBaselineValidator {
    pub rules: Vec<Rc<dyn AdaptiveIdentityBaselineRule>>,
}

impl AdaptiveIdentityBaselineValidator {
    pub fn validate(&self, item: &AdaptiveIdentityBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveIdentityBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveIdentityBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_identity_baseline_validator() -> AdaptiveIdentityBaselineValidator {
    AdaptiveIdentityBaselineValidator {
        rules: vec![
            Rc::new(AdaptiveIdentityBaselinePinnedGuardRule { limit: 80 }) as Rc<dyn AdaptiveIdentityBaselineRule>,
            Rc::new(AdaptiveIdentityBaselineNameLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveIdentityBaselineGraceDaysRangeRule { min: 1000, max: 86400 }),
            Rc::new(AdaptiveIdentityBaselineWeightRangeRule { min: 2.2, max: 4803.5 }),
            Rc::new(AdaptiveIdentityBaselineCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_identity_baseline_validator();
        let item = new_adaptive_identity_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AdaptiveIdentityBaselineNameLengthRule { max_length: 16 };
        let mut item = new_adaptive_identity_baseline("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = AdaptiveIdentityBaselineGraceDaysRangeRule { min: 80, max: 255 };
        let mut item = new_adaptive_identity_baseline("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = AdaptiveIdentityBaselineWeightRangeRule { min: 2.2, max: 3591.9 };
        let mut item = new_adaptive_identity_baseline("x", "y");
        item.weight = 3593.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveIdentityBaselineCodeLengthRule { max_length: 16 };
        let mut item = new_adaptive_identity_baseline("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
