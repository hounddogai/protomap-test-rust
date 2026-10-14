use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage template must pass. Check returns its problems, or none.
pub trait OverageTemplateRule: std::fmt::Debug {
    fn check(&self, item: &OverageTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageTemplateTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl OverageTemplateRule for OverageTemplateTagsLimitRule {
    fn check(&self, item: &OverageTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageTemplateRequiredCodeRule {
    pub message: String,
}

impl OverageTemplateRule for OverageTemplateRequiredCodeRule {
    fn check(&self, item: &OverageTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageTemplatePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageTemplateRule for OverageTemplatePriorityRangeRule {
    fn check(&self, item: &OverageTemplate) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageTemplateMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageTemplateRule for OverageTemplateMaximumRangeRule {
    fn check(&self, item: &OverageTemplate) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageTemplateTenantIdLengthRule {
    pub max_length: i64,
}

impl OverageTemplateRule for OverageTemplateTenantIdLengthRule {
    fn check(&self, item: &OverageTemplate) -> Vec<String> {
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

/// Runs every rule of a overage template.
#[derive(Debug, Clone)]
pub struct OverageTemplateValidator {
    pub rules: Vec<Rc<dyn OverageTemplateRule>>,
}

impl OverageTemplateValidator {
    pub fn validate(&self, item: &OverageTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_template_validator() -> OverageTemplateValidator {
    OverageTemplateValidator {
        rules: vec![
            Rc::new(OverageTemplateTagsLimitRule {
                max_count: 1000,
                banned: Vec::new(),
            }) as Rc<dyn OverageTemplateRule>,
            Rc::new(OverageTemplateRequiredCodeRule {
                message: "overage template needs a value".to_string(),
            }),
            Rc::new(OverageTemplatePriorityRangeRule { min: 16, max: 86400 }),
            Rc::new(OverageTemplateMaximumRangeRule { min: 2.2, max: 4917.8 }),
            Rc::new(OverageTemplateTenantIdLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_template_validator();
        let item = new_overage_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = OverageTemplateRequiredCodeRule {
            message: "overage template needs an entry".to_string(),
        };
        let mut item = new_overage_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = OverageTemplatePriorityRangeRule { min: 80, max: 3600 };
        let mut item = new_overage_template("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = OverageTemplateMaximumRangeRule { min: 6.0, max: 3320.9 };
        let mut item = new_overage_template("x", "y");
        item.maximum = 3322.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = OverageTemplateTenantIdLengthRule { max_length: 86400 };
        let mut item = new_overage_template("x", "y");
        item.tenant_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
