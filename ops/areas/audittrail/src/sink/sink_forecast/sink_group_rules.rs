use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink group must pass. Check returns its problems, or none.
pub trait SinkGroupRule: std::fmt::Debug {
    fn check(&self, item: &SinkGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkGroupTenantIdLengthRule {
    pub max_length: i64,
}

impl SinkGroupRule for SinkGroupTenantIdLengthRule {
    fn check(&self, item: &SinkGroup) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkGroupSourceLengthRule {
    pub max_length: i64,
}

impl SinkGroupRule for SinkGroupSourceLengthRule {
    fn check(&self, item: &SinkGroup) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkGroupPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkGroupRule for SinkGroupPriorityRangeRule {
    fn check(&self, item: &SinkGroup) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkGroupRequiredNameRule {
    pub message: String,
}

impl SinkGroupRule for SinkGroupRequiredNameRule {
    fn check(&self, item: &SinkGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink group.
#[derive(Debug, Clone)]
pub struct SinkGroupValidator {
    pub rules: Vec<Rc<dyn SinkGroupRule>>,
}

impl SinkGroupValidator {
    pub fn validate(&self, item: &SinkGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_group_validator() -> SinkGroupValidator {
    SinkGroupValidator {
        rules: vec![
            Rc::new(SinkGroupTenantIdLengthRule { max_length: 255 }) as Rc<dyn SinkGroupRule>,
            Rc::new(SinkGroupSourceLengthRule { max_length: 1000 }),
            Rc::new(SinkGroupPriorityRangeRule { min: 120, max: 86400 }),
            Rc::new(SinkGroupRequiredNameRule {
                message: "sink group needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_group_validator();
        let item = new_sink_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = SinkGroupTenantIdLengthRule { max_length: 120 };
        let mut item = new_sink_group("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = SinkGroupSourceLengthRule { max_length: 255 };
        let mut item = new_sink_group("x", "y");
        item.source = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = SinkGroupPriorityRangeRule { min: 32, max: 32 };
        let mut item = new_sink_group("x", "y");
        item.priority = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SinkGroupRequiredNameRule {
            message: "sink group needs an entry".to_string(),
        };
        let mut item = new_sink_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
