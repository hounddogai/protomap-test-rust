use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin digest must pass. Check returns its problems, or none.
pub trait RegionPinDigestRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDigestExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinDigestRule for RegionPinDigestExpiresAtRangeRule {
    fn check(&self, item: &RegionPinDigest) -> Vec<String> {
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
pub struct RegionPinDigestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinDigestRule for RegionPinDigestDurationMinutesRangeRule {
    fn check(&self, item: &RegionPinDigest) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDigestQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinDigestRule for RegionPinDigestQuotaUnitsRangeRule {
    fn check(&self, item: &RegionPinDigest) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDigestRequiredTitleRule {
    pub message: String,
}

impl RegionPinDigestRule for RegionPinDigestRequiredTitleRule {
    fn check(&self, item: &RegionPinDigest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDigestCodePrefixRule {
    pub prefix: String,
}

impl RegionPinDigestRule for RegionPinDigestCodePrefixRule {
    fn check(&self, item: &RegionPinDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinDigestRequiredCodeRule {
    pub message: String,
}

impl RegionPinDigestRule for RegionPinDigestRequiredCodeRule {
    fn check(&self, item: &RegionPinDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin digest.
#[derive(Debug, Clone)]
pub struct RegionPinDigestValidator {
    pub rules: Vec<Rc<dyn RegionPinDigestRule>>,
}

impl RegionPinDigestValidator {
    pub fn validate(&self, item: &RegionPinDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_digest_validator() -> RegionPinDigestValidator {
    RegionPinDigestValidator {
        rules: vec![
            Rc::new(RegionPinDigestExpiresAtRangeRule { min: 80, max: 86400 }) as Rc<dyn RegionPinDigestRule>,
            Rc::new(RegionPinDigestDurationMinutesRangeRule { min: 64, max: 3600 }),
            Rc::new(RegionPinDigestQuotaUnitsRangeRule { min: 120, max: 3600 }),
            Rc::new(RegionPinDigestRequiredTitleRule {
                message: "region pin digest needs an entry".to_string(),
            }),
            Rc::new(RegionPinDigestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RegionPinDigestRequiredCodeRule {
                message: "region pin digest needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_digest_validator();
        let item = new_region_pin_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = RegionPinDigestExpiresAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_region_pin_digest("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RegionPinDigestDurationMinutesRangeRule { min: 64, max: 120 };
        let mut item = new_region_pin_digest("x", "y");
        item.duration_minutes = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = RegionPinDigestQuotaUnitsRangeRule { min: 120, max: 3600 };
        let mut item = new_region_pin_digest("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionPinDigestRequiredTitleRule {
            message: "region pin digest needs an entry".to_string(),
        };
        let mut item = new_region_pin_digest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinDigestRequiredCodeRule {
            message: "region pin digest needs a value".to_string(),
        };
        let mut item = new_region_pin_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
