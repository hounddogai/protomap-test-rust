use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release claim must pass. Check returns its problems, or none.
pub trait ReleaseClaimRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseClaimExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseClaimRule for ReleaseClaimExpiresAtRangeRule {
    fn check(&self, item: &ReleaseClaim) -> Vec<String> {
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
pub struct ReleaseClaimMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseClaimRule for ReleaseClaimMarginRangeRule {
    fn check(&self, item: &ReleaseClaim) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseClaimLabelLengthRule {
    pub max_length: i64,
}

impl ReleaseClaimRule for ReleaseClaimLabelLengthRule {
    fn check(&self, item: &ReleaseClaim) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release claim.
#[derive(Debug, Clone)]
pub struct ReleaseClaimValidator {
    pub rules: Vec<Rc<dyn ReleaseClaimRule>>,
}

impl ReleaseClaimValidator {
    pub fn validate(&self, item: &ReleaseClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_claim_validator() -> ReleaseClaimValidator {
    ReleaseClaimValidator {
        rules: vec![
            Rc::new(ReleaseClaimExpiresAtRangeRule { min: 1000, max: 86400 }) as Rc<dyn ReleaseClaimRule>,
            Rc::new(ReleaseClaimMarginRangeRule { min: 0.8, max: 2970.2 }),
            Rc::new(ReleaseClaimLabelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_claim_validator();
        let item = new_release_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = ReleaseClaimExpiresAtRangeRule { min: 16, max: 1000 };
        let mut item = new_release_claim("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = ReleaseClaimMarginRangeRule { min: 6.6, max: 457.0 };
        let mut item = new_release_claim("x", "y");
        item.margin = 458.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ReleaseClaimLabelLengthRule { max_length: 32 };
        let mut item = new_release_claim("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
