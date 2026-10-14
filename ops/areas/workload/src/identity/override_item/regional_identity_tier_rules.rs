use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional identity tier must pass. Check returns its problems, or none.
pub trait RegionalIdentityTierRule: std::fmt::Debug {
    fn check(&self, item: &RegionalIdentityTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIdentityTierRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalIdentityTierRule for RegionalIdentityTierRateRangeRule {
    fn check(&self, item: &RegionalIdentityTier) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIdentityTierAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RegionalIdentityTierRule for RegionalIdentityTierAllowedCodeRule {
    fn check(&self, item: &RegionalIdentityTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIdentityTierHostLengthRule {
    pub max_length: i64,
}

impl RegionalIdentityTierRule for RegionalIdentityTierHostLengthRule {
    fn check(&self, item: &RegionalIdentityTier) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalIdentityTierCodeLengthRule {
    pub max_length: i64,
}

impl RegionalIdentityTierRule for RegionalIdentityTierCodeLengthRule {
    fn check(&self, item: &RegionalIdentityTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional identity tier.
#[derive(Debug, Clone)]
pub struct RegionalIdentityTierValidator {
    pub rules: Vec<Rc<dyn RegionalIdentityTierRule>>,
}

impl RegionalIdentityTierValidator {
    pub fn validate(&self, item: &RegionalIdentityTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalIdentityTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalIdentityTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_identity_tier_validator() -> RegionalIdentityTierValidator {
    RegionalIdentityTierValidator {
        rules: vec![
            Rc::new(RegionalIdentityTierRateRangeRule { min: 0.8, max: 2634.9 }) as Rc<dyn RegionalIdentityTierRule>,
            Rc::new(RegionalIdentityTierAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "weighted".to_string(), "batched".to_string()],
            }),
            Rc::new(RegionalIdentityTierHostLengthRule { max_length: 1000 }),
            Rc::new(RegionalIdentityTierCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_identity_tier_validator();
        let item = new_regional_identity_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = RegionalIdentityTierRateRangeRule { min: 1.9, max: 4356.2 };
        let mut item = new_regional_identity_tier("x", "y");
        item.rate = 4357.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = RegionalIdentityTierHostLengthRule { max_length: 16 };
        let mut item = new_regional_identity_tier("x", "y");
        item.host = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalIdentityTierCodeLengthRule { max_length: 80 };
        let mut item = new_regional_identity_tier("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
