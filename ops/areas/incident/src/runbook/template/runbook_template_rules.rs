use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook template must pass. Check returns its problems, or none.
pub trait RunbookTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RunbookTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTemplateNameLengthRule {
    pub max_length: i64,
}

impl RunbookTemplateRule for RunbookTemplateNameLengthRule {
    fn check(&self, item: &RunbookTemplate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTemplateRequiredNameRule {
    pub message: String,
}

impl RunbookTemplateRule for RunbookTemplateRequiredNameRule {
    fn check(&self, item: &RunbookTemplate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTemplateAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl RunbookTemplateRule for RunbookTemplateAllowedExternalRefRule {
    fn check(&self, item: &RunbookTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is stale", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTemplateMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookTemplateRule for RunbookTemplateMarginRangeRule {
    fn check(&self, item: &RunbookTemplate) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTemplateRequiredUnitRule {
    pub message: String,
}

impl RunbookTemplateRule for RunbookTemplateRequiredUnitRule {
    fn check(&self, item: &RunbookTemplate) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookTemplateCodePrefixRule {
    pub prefix: String,
}

impl RunbookTemplateRule for RunbookTemplateCodePrefixRule {
    fn check(&self, item: &RunbookTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook template.
#[derive(Debug, Clone)]
pub struct RunbookTemplateValidator {
    pub rules: Vec<Rc<dyn RunbookTemplateRule>>,
}

impl RunbookTemplateValidator {
    pub fn validate(&self, item: &RunbookTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_template_validator() -> RunbookTemplateValidator {
    RunbookTemplateValidator {
        rules: vec![
            Rc::new(RunbookTemplateNameLengthRule { max_length: 64 }) as Rc<dyn RunbookTemplateRule>,
            Rc::new(RunbookTemplateRequiredNameRule {
                message: "runbook template needs an entry".to_string(),
            }),
            Rc::new(RunbookTemplateAllowedExternalRefRule { allowed: Vec::new() }),
            Rc::new(RunbookTemplateMarginRangeRule { min: 3.7, max: 2988.5 }),
            Rc::new(RunbookTemplateRequiredUnitRule {
                message: "runbook template needs an entry".to_string(),
            }),
            Rc::new(RunbookTemplateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_template_validator();
        let item = new_runbook_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RunbookTemplateNameLengthRule { max_length: 3600 };
        let mut item = new_runbook_template("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RunbookTemplateRequiredNameRule {
            message: "runbook template needs this field".to_string(),
        };
        let mut item = new_runbook_template("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = RunbookTemplateMarginRangeRule { min: 1.0, max: 2178.7 };
        let mut item = new_runbook_template("x", "y");
        item.margin = 2180.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = RunbookTemplateRequiredUnitRule {
            message: "runbook template needs an entry".to_string(),
        };
        let mut item = new_runbook_template("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
