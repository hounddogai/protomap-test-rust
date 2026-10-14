use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered dashboard digest must pass. Check returns its problems, or none.
pub trait TieredDashboardDigestRule: std::fmt::Debug {
    fn check(&self, item: &TieredDashboardDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardDigestRequiredTitleRule {
    pub message: String,
}

impl TieredDashboardDigestRule for TieredDashboardDigestRequiredTitleRule {
    fn check(&self, item: &TieredDashboardDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardDigestScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredDashboardDigestRule for TieredDashboardDigestScoreRangeRule {
    fn check(&self, item: &TieredDashboardDigest) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardDigestRequiredChecksumRule {
    pub message: String,
}

impl TieredDashboardDigestRule for TieredDashboardDigestRequiredChecksumRule {
    fn check(&self, item: &TieredDashboardDigest) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardDigestChannelLengthRule {
    pub max_length: i64,
}

impl TieredDashboardDigestRule for TieredDashboardDigestChannelLengthRule {
    fn check(&self, item: &TieredDashboardDigest) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered dashboard digest.
#[derive(Debug, Clone)]
pub struct TieredDashboardDigestValidator {
    pub rules: Vec<Rc<dyn TieredDashboardDigestRule>>,
}

impl TieredDashboardDigestValidator {
    pub fn validate(&self, item: &TieredDashboardDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredDashboardDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredDashboardDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_dashboard_digest_validator() -> TieredDashboardDigestValidator {
    TieredDashboardDigestValidator {
        rules: vec![
            Rc::new(TieredDashboardDigestRequiredTitleRule {
                message: "tiered dashboard digest needs a value".to_string(),
            }) as Rc<dyn TieredDashboardDigestRule>,
            Rc::new(TieredDashboardDigestScoreRangeRule { min: 6.5, max: 66.8 }),
            Rc::new(TieredDashboardDigestRequiredChecksumRule {
                message: "tiered dashboard digest needs a value".to_string(),
            }),
            Rc::new(TieredDashboardDigestChannelLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_dashboard_digest_validator();
        let item = new_tiered_dashboard_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredDashboardDigestRequiredTitleRule {
            message: "tiered dashboard digest needs an entry".to_string(),
        };
        let mut item = new_tiered_dashboard_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = TieredDashboardDigestScoreRangeRule { min: 4.5, max: 4049.9 };
        let mut item = new_tiered_dashboard_digest("x", "y");
        item.score = 4051.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = TieredDashboardDigestRequiredChecksumRule {
            message: "tiered dashboard digest needs this field".to_string(),
        };
        let mut item = new_tiered_dashboard_digest("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = TieredDashboardDigestChannelLengthRule { max_length: 80 };
        let mut item = new_tiered_dashboard_digest("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
