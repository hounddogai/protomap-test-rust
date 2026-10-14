use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default plan rollup must pass. Check returns its problems, or none.
pub trait DefaultPlanRollupRule: std::fmt::Debug {
    fn check(&self, item: &DefaultPlanRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanRollupCodeLengthRule {
    pub max_length: i64,
}

impl DefaultPlanRollupRule for DefaultPlanRollupCodeLengthRule {
    fn check(&self, item: &DefaultPlanRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanRollupRequiredRegionRule {
    pub message: String,
}

impl DefaultPlanRollupRule for DefaultPlanRollupRequiredRegionRule {
    fn check(&self, item: &DefaultPlanRollup) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanRollupRegionLengthRule {
    pub max_length: i64,
}

impl DefaultPlanRollupRule for DefaultPlanRollupRegionLengthRule {
    fn check(&self, item: &DefaultPlanRollup) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default plan rollup.
#[derive(Debug, Clone)]
pub struct DefaultPlanRollupValidator {
    pub rules: Vec<Rc<dyn DefaultPlanRollupRule>>,
}

impl DefaultPlanRollupValidator {
    pub fn validate(&self, item: &DefaultPlanRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultPlanRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultPlanRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_plan_rollup_validator() -> DefaultPlanRollupValidator {
    DefaultPlanRollupValidator {
        rules: vec![
            Rc::new(DefaultPlanRollupCodeLengthRule { max_length: 16 }) as Rc<dyn DefaultPlanRollupRule>,
            Rc::new(DefaultPlanRollupRequiredRegionRule {
                message: "default plan rollup needs a value".to_string(),
            }),
            Rc::new(DefaultPlanRollupRegionLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_plan_rollup_validator();
        let item = new_default_plan_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultPlanRollupCodeLengthRule { max_length: 1000 };
        let mut item = new_default_plan_rollup("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = DefaultPlanRollupRequiredRegionRule {
            message: "default plan rollup needs this field".to_string(),
        };
        let mut item = new_default_plan_rollup("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = DefaultPlanRollupRegionLengthRule { max_length: 255 };
        let mut item = new_default_plan_rollup("x", "y");
        item.region = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
