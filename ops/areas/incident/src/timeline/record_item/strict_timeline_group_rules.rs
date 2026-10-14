use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict timeline group must pass. Check returns its problems, or none.
pub trait StrictTimelineGroupRule: std::fmt::Debug {
    fn check(&self, item: &StrictTimelineGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineGroupRequiredHostRule {
    pub message: String,
}

impl StrictTimelineGroupRule for StrictTimelineGroupRequiredHostRule {
    fn check(&self, item: &StrictTimelineGroup) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineGroupBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictTimelineGroupRule for StrictTimelineGroupBurstRangeRule {
    fn check(&self, item: &StrictTimelineGroup) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineGroupVisibleGuardRule {
    pub limit: i64,
}

impl StrictTimelineGroupRule for StrictTimelineGroupVisibleGuardRule {
    fn check(&self, item: &StrictTimelineGroup) -> Vec<String> {
        if item.visible && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while visible", item.burst, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineGroupRequiredCipherRule {
    pub message: String,
}

impl StrictTimelineGroupRule for StrictTimelineGroupRequiredCipherRule {
    fn check(&self, item: &StrictTimelineGroup) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineGroupCodePrefixRule {
    pub prefix: String,
}

impl StrictTimelineGroupRule for StrictTimelineGroupCodePrefixRule {
    fn check(&self, item: &StrictTimelineGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTimelineGroupVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictTimelineGroupRule for StrictTimelineGroupVersionRangeRule {
    fn check(&self, item: &StrictTimelineGroup) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict timeline group.
#[derive(Debug, Clone)]
pub struct StrictTimelineGroupValidator {
    pub rules: Vec<Rc<dyn StrictTimelineGroupRule>>,
}

impl StrictTimelineGroupValidator {
    pub fn validate(&self, item: &StrictTimelineGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictTimelineGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictTimelineGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_timeline_group_validator() -> StrictTimelineGroupValidator {
    StrictTimelineGroupValidator {
        rules: vec![
            Rc::new(StrictTimelineGroupRequiredHostRule {
                message: "strict timeline group needs an entry".to_string(),
            }) as Rc<dyn StrictTimelineGroupRule>,
            Rc::new(StrictTimelineGroupBurstRangeRule { min: 120, max: 86400 }),
            Rc::new(StrictTimelineGroupVisibleGuardRule { limit: 86400 }),
            Rc::new(StrictTimelineGroupRequiredCipherRule {
                message: "strict timeline group needs this field".to_string(),
            }),
            Rc::new(StrictTimelineGroupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(StrictTimelineGroupVersionRangeRule { min: 16, max: 1000 }),
        ],
    }
}
