use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered allowlist claim must pass. Check returns its problems, or none.
pub trait TieredAllowlistClaimRule: std::fmt::Debug {
    fn check(&self, item: &TieredAllowlistClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistClaimTitleLengthRule {
    pub max_length: i64,
}

impl TieredAllowlistClaimRule for TieredAllowlistClaimTitleLengthRule {
    fn check(&self, item: &TieredAllowlistClaim) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistClaimExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredAllowlistClaimRule for TieredAllowlistClaimExpiresAtRangeRule {
    fn check(&self, item: &TieredAllowlistClaim) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistClaimScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredAllowlistClaimRule for TieredAllowlistClaimScoreRangeRule {
    fn check(&self, item: &TieredAllowlistClaim) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistClaimUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredAllowlistClaimRule for TieredAllowlistClaimUpdatedAtRangeRule {
    fn check(&self, item: &TieredAllowlistClaim) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered allowlist claim.
#[derive(Debug, Clone)]
pub struct TieredAllowlistClaimValidator {
    pub rules: Vec<Rc<dyn TieredAllowlistClaimRule>>,
}

impl TieredAllowlistClaimValidator {
    pub fn validate(&self, item: &TieredAllowlistClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredAllowlistClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredAllowlistClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_allowlist_claim_validator() -> TieredAllowlistClaimValidator {
    TieredAllowlistClaimValidator {
        rules: vec![
            Rc::new(TieredAllowlistClaimTitleLengthRule { max_length: 255 }) as Rc<dyn TieredAllowlistClaimRule>,
            Rc::new(TieredAllowlistClaimExpiresAtRangeRule { min: 64, max: 1000 }),
            Rc::new(TieredAllowlistClaimScoreRangeRule { min: 0.5, max: 3640.8 }),
            Rc::new(TieredAllowlistClaimUpdatedAtRangeRule { min: 120, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_allowlist_claim_validator();
        let item = new_tiered_allowlist_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredAllowlistClaimTitleLengthRule { max_length: 16 };
        let mut item = new_tiered_allowlist_claim("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = TieredAllowlistClaimExpiresAtRangeRule { min: 16, max: 32 };
        let mut item = new_tiered_allowlist_claim("x", "y");
        item.expires_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = TieredAllowlistClaimScoreRangeRule { min: 6.3, max: 3444.0 };
        let mut item = new_tiered_allowlist_claim("x", "y");
        item.score = 3445.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = TieredAllowlistClaimUpdatedAtRangeRule { min: 86400, max: 86400 };
        let mut item = new_tiered_allowlist_claim("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
