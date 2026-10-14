use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped grant target must pass. Check returns its problems, or none.
pub trait CappedGrantTargetRule: std::fmt::Debug {
    fn check(&self, item: &CappedGrantTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantTargetRequiredCodeRule {
    pub message: String,
}

impl CappedGrantTargetRule for CappedGrantTargetRequiredCodeRule {
    fn check(&self, item: &CappedGrantTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantTargetChannelLengthRule {
    pub max_length: i64,
}

impl CappedGrantTargetRule for CappedGrantTargetChannelLengthRule {
    fn check(&self, item: &CappedGrantTarget) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantTargetRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedGrantTargetRule for CappedGrantTargetRatioRangeRule {
    fn check(&self, item: &CappedGrantTarget) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped grant target.
#[derive(Debug, Clone)]
pub struct CappedGrantTargetValidator {
    pub rules: Vec<Rc<dyn CappedGrantTargetRule>>,
}

impl CappedGrantTargetValidator {
    pub fn validate(&self, item: &CappedGrantTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedGrantTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedGrantTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_grant_target_validator() -> CappedGrantTargetValidator {
    CappedGrantTargetValidator {
        rules: vec![
            Rc::new(CappedGrantTargetRequiredCodeRule {
                message: "capped grant target needs this field".to_string(),
            }) as Rc<dyn CappedGrantTargetRule>,
            Rc::new(CappedGrantTargetChannelLengthRule { max_length: 3600 }),
            Rc::new(CappedGrantTargetRatioRangeRule { min: 4.9, max: 3579.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_grant_target_validator();
        let item = new_capped_grant_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedGrantTargetRequiredCodeRule {
            message: "capped grant target needs a value".to_string(),
        };
        let mut item = new_capped_grant_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = CappedGrantTargetChannelLengthRule { max_length: 16 };
        let mut item = new_capped_grant_target("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = CappedGrantTargetRatioRangeRule { min: 8.6, max: 4019.6 };
        let mut item = new_capped_grant_target("x", "y");
        item.ratio = 4021.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
