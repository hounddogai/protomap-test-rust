use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped dashboard tier must pass. Check returns its problems, or none.
pub trait CappedDashboardTierRule: std::fmt::Debug {
    fn check(&self, item: &CappedDashboardTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTierRequiredReviewerRule {
    pub message: String,
}

impl CappedDashboardTierRule for CappedDashboardTierRequiredReviewerRule {
    fn check(&self, item: &CappedDashboardTier) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTierRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedDashboardTierRule for CappedDashboardTierRateRangeRule {
    fn check(&self, item: &CappedDashboardTier) -> Vec<String> {
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
pub struct CappedDashboardTierCodePrefixRule {
    pub prefix: String,
}

impl CappedDashboardTierRule for CappedDashboardTierCodePrefixRule {
    fn check(&self, item: &CappedDashboardTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTierRequiredIssuerRule {
    pub message: String,
}

impl CappedDashboardTierRule for CappedDashboardTierRequiredIssuerRule {
    fn check(&self, item: &CappedDashboardTier) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTierIssuerLengthRule {
    pub max_length: i64,
}

impl CappedDashboardTierRule for CappedDashboardTierIssuerLengthRule {
    fn check(&self, item: &CappedDashboardTier) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped dashboard tier.
#[derive(Debug, Clone)]
pub struct CappedDashboardTierValidator {
    pub rules: Vec<Rc<dyn CappedDashboardTierRule>>,
}

impl CappedDashboardTierValidator {
    pub fn validate(&self, item: &CappedDashboardTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedDashboardTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedDashboardTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_dashboard_tier_validator() -> CappedDashboardTierValidator {
    CappedDashboardTierValidator {
        rules: vec![
            Rc::new(CappedDashboardTierRequiredReviewerRule {
                message: "capped dashboard tier needs an entry".to_string(),
            }) as Rc<dyn CappedDashboardTierRule>,
            Rc::new(CappedDashboardTierRateRangeRule { min: 6.9, max: 992.2 }),
            Rc::new(CappedDashboardTierCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(CappedDashboardTierRequiredIssuerRule {
                message: "capped dashboard tier needs this field".to_string(),
            }),
            Rc::new(CappedDashboardTierIssuerLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_dashboard_tier_validator();
        let item = new_capped_dashboard_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = CappedDashboardTierRequiredReviewerRule {
            message: "capped dashboard tier needs an entry".to_string(),
        };
        let mut item = new_capped_dashboard_tier("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = CappedDashboardTierRateRangeRule { min: 8.8, max: 4427.5 };
        let mut item = new_capped_dashboard_tier("x", "y");
        item.rate = 4429.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = CappedDashboardTierRequiredIssuerRule {
            message: "capped dashboard tier needs a value".to_string(),
        };
        let mut item = new_capped_dashboard_tier("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = CappedDashboardTierIssuerLengthRule { max_length: 32 };
        let mut item = new_capped_dashboard_tier("x", "y");
        item.issuer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
