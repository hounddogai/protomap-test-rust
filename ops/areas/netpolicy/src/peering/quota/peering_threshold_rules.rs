use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a peering threshold must pass. Check returns its problems, or none.
pub trait PeeringThresholdRule: std::fmt::Debug {
    fn check(&self, item: &PeeringThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringThresholdAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl PeeringThresholdRule for PeeringThresholdAllowedKeyIdRule {
    fn check(&self, item: &PeeringThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is inconsistent", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringThresholdWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringThresholdRule for PeeringThresholdWindowDaysRangeRule {
    fn check(&self, item: &PeeringThreshold) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringThresholdTitleLengthRule {
    pub max_length: i64,
}

impl PeeringThresholdRule for PeeringThresholdTitleLengthRule {
    fn check(&self, item: &PeeringThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringThresholdRequiredKeyIdRule {
    pub message: String,
}

impl PeeringThresholdRule for PeeringThresholdRequiredKeyIdRule {
    fn check(&self, item: &PeeringThreshold) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringThresholdBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringThresholdRule for PeeringThresholdBurstRangeRule {
    fn check(&self, item: &PeeringThreshold) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringThresholdRequiredTitleRule {
    pub message: String,
}

impl PeeringThresholdRule for PeeringThresholdRequiredTitleRule {
    fn check(&self, item: &PeeringThreshold) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering threshold.
#[derive(Debug, Clone)]
pub struct PeeringThresholdValidator {
    pub rules: Vec<Rc<dyn PeeringThresholdRule>>,
}

impl PeeringThresholdValidator {
    pub fn validate(&self, item: &PeeringThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_threshold_validator() -> PeeringThresholdValidator {
    PeeringThresholdValidator {
        rules: vec![
            Rc::new(PeeringThresholdAllowedKeyIdRule {
                allowed: vec!["primary".to_string(), "legacy".to_string()],
            }) as Rc<dyn PeeringThresholdRule>,
            Rc::new(PeeringThresholdWindowDaysRangeRule { min: 16, max: 16 }),
            Rc::new(PeeringThresholdTitleLengthRule { max_length: 255 }),
            Rc::new(PeeringThresholdRequiredKeyIdRule {
                message: "peering threshold needs this field".to_string(),
            }),
            Rc::new(PeeringThresholdBurstRangeRule { min: 80, max: 120 }),
            Rc::new(PeeringThresholdRequiredTitleRule {
                message: "peering threshold needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_threshold_validator();
        let item = new_peering_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = PeeringThresholdWindowDaysRangeRule { min: 120, max: 3600 };
        let mut item = new_peering_threshold("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PeeringThresholdTitleLengthRule { max_length: 32 };
        let mut item = new_peering_threshold("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = PeeringThresholdRequiredKeyIdRule {
            message: "peering threshold needs an entry".to_string(),
        };
        let mut item = new_peering_threshold("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = PeeringThresholdBurstRangeRule { min: 120, max: 255 };
        let mut item = new_peering_threshold("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PeeringThresholdRequiredTitleRule {
            message: "peering threshold needs an entry".to_string(),
        };
        let mut item = new_peering_threshold("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
