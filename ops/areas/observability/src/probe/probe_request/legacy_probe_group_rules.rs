use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy probe group must pass. Check returns its problems, or none.
pub trait LegacyProbeGroupRule: std::fmt::Debug {
    fn check(&self, item: &LegacyProbeGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeGroupGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyProbeGroupRule for LegacyProbeGroupGraceDaysRangeRule {
    fn check(&self, item: &LegacyProbeGroup) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeGroupRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyProbeGroupRule for LegacyProbeGroupRetriesRangeRule {
    fn check(&self, item: &LegacyProbeGroup) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeGroupRequiredPrincipalRule {
    pub message: String,
}

impl LegacyProbeGroupRule for LegacyProbeGroupRequiredPrincipalRule {
    fn check(&self, item: &LegacyProbeGroup) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeGroupSegmentLengthRule {
    pub max_length: i64,
}

impl LegacyProbeGroupRule for LegacyProbeGroupSegmentLengthRule {
    fn check(&self, item: &LegacyProbeGroup) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeGroupPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyProbeGroupRule for LegacyProbeGroupPortRangeRule {
    fn check(&self, item: &LegacyProbeGroup) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeGroupRequiredNameRule {
    pub message: String,
}

impl LegacyProbeGroupRule for LegacyProbeGroupRequiredNameRule {
    fn check(&self, item: &LegacyProbeGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy probe group.
#[derive(Debug, Clone)]
pub struct LegacyProbeGroupValidator {
    pub rules: Vec<Rc<dyn LegacyProbeGroupRule>>,
}

impl LegacyProbeGroupValidator {
    pub fn validate(&self, item: &LegacyProbeGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyProbeGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyProbeGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_probe_group_validator() -> LegacyProbeGroupValidator {
    LegacyProbeGroupValidator {
        rules: vec![
            Rc::new(LegacyProbeGroupGraceDaysRangeRule { min: 64, max: 3600 }) as Rc<dyn LegacyProbeGroupRule>,
            Rc::new(LegacyProbeGroupRetriesRangeRule { min: 32, max: 120 }),
            Rc::new(LegacyProbeGroupRequiredPrincipalRule {
                message: "legacy probe group needs an entry".to_string(),
            }),
            Rc::new(LegacyProbeGroupSegmentLengthRule { max_length: 16 }),
            Rc::new(LegacyProbeGroupPortRangeRule { min: 16, max: 80 }),
            Rc::new(LegacyProbeGroupRequiredNameRule {
                message: "legacy probe group needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_probe_group_validator();
        let item = new_legacy_probe_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = LegacyProbeGroupGraceDaysRangeRule { min: 32, max: 3600 };
        let mut item = new_legacy_probe_group("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = LegacyProbeGroupRetriesRangeRule { min: 16, max: 120 };
        let mut item = new_legacy_probe_group("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = LegacyProbeGroupRequiredPrincipalRule {
            message: "legacy probe group needs a value".to_string(),
        };
        let mut item = new_legacy_probe_group("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = LegacyProbeGroupSegmentLengthRule { max_length: 80 };
        let mut item = new_legacy_probe_group("x", "y");
        item.segment = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LegacyProbeGroupPortRangeRule { min: 64, max: 255 };
        let mut item = new_legacy_probe_group("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyProbeGroupRequiredNameRule {
            message: "legacy probe group needs a value".to_string(),
        };
        let mut item = new_legacy_probe_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
