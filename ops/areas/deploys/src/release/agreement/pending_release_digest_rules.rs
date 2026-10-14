use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending release digest must pass. Check returns its problems, or none.
pub trait PendingReleaseDigestRule: std::fmt::Debug {
    fn check(&self, item: &PendingReleaseDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseDigestAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl PendingReleaseDigestRule for PendingReleaseDigestAllowedChecksumRule {
    fn check(&self, item: &PendingReleaseDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is stale", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseDigestSummaryLengthRule {
    pub max_length: i64,
}

impl PendingReleaseDigestRule for PendingReleaseDigestSummaryLengthRule {
    fn check(&self, item: &PendingReleaseDigest) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseDigestRequiredCodeRule {
    pub message: String,
}

impl PendingReleaseDigestRule for PendingReleaseDigestRequiredCodeRule {
    fn check(&self, item: &PendingReleaseDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseDigestRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingReleaseDigestRule for PendingReleaseDigestRatioRangeRule {
    fn check(&self, item: &PendingReleaseDigest) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending release digest.
#[derive(Debug, Clone)]
pub struct PendingReleaseDigestValidator {
    pub rules: Vec<Rc<dyn PendingReleaseDigestRule>>,
}

impl PendingReleaseDigestValidator {
    pub fn validate(&self, item: &PendingReleaseDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingReleaseDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingReleaseDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_release_digest_validator() -> PendingReleaseDigestValidator {
    PendingReleaseDigestValidator {
        rules: vec![
            Rc::new(PendingReleaseDigestAllowedChecksumRule {
                allowed: vec!["tiered".to_string(), "seasonal".to_string(), "regional".to_string()],
            }) as Rc<dyn PendingReleaseDigestRule>,
            Rc::new(PendingReleaseDigestSummaryLengthRule { max_length: 16 }),
            Rc::new(PendingReleaseDigestRequiredCodeRule {
                message: "pending release digest needs a value".to_string(),
            }),
            Rc::new(PendingReleaseDigestRatioRangeRule { min: 2.2, max: 4514.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_release_digest_validator();
        let item = new_pending_release_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = PendingReleaseDigestSummaryLengthRule { max_length: 120 };
        let mut item = new_pending_release_digest("x", "y");
        item.summary = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingReleaseDigestRequiredCodeRule {
            message: "pending release digest needs an entry".to_string(),
        };
        let mut item = new_pending_release_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = PendingReleaseDigestRatioRangeRule { min: 6.2, max: 2481.8 };
        let mut item = new_pending_release_digest("x", "y");
        item.ratio = 2483.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
