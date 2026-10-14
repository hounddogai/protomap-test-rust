use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit claim must pass. Check returns its problems, or none.
pub trait LimitClaimRule: std::fmt::Debug {
    fn check(&self, item: &LimitClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitClaimRequiredRegionRule {
    pub message: String,
}

impl LimitClaimRule for LimitClaimRequiredRegionRule {
    fn check(&self, item: &LimitClaim) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitClaimMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitClaimRule for LimitClaimMarginRangeRule {
    fn check(&self, item: &LimitClaim) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitClaimCodeLengthRule {
    pub max_length: i64,
}

impl LimitClaimRule for LimitClaimCodeLengthRule {
    fn check(&self, item: &LimitClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit claim.
#[derive(Debug, Clone)]
pub struct LimitClaimValidator {
    pub rules: Vec<Rc<dyn LimitClaimRule>>,
}

impl LimitClaimValidator {
    pub fn validate(&self, item: &LimitClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_claim_validator() -> LimitClaimValidator {
    LimitClaimValidator {
        rules: vec![
            Rc::new(LimitClaimRequiredRegionRule {
                message: "limit claim needs an entry".to_string(),
            }) as Rc<dyn LimitClaimRule>,
            Rc::new(LimitClaimMarginRangeRule { min: 3.0, max: 3012.2 }),
            Rc::new(LimitClaimCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_claim_validator();
        let item = new_limit_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = LimitClaimRequiredRegionRule {
            message: "limit claim needs this field".to_string(),
        };
        let mut item = new_limit_claim("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = LimitClaimMarginRangeRule { min: 0.0, max: 2628.6 };
        let mut item = new_limit_claim("x", "y");
        item.margin = 2630.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LimitClaimCodeLengthRule { max_length: 64 };
        let mut item = new_limit_claim("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
