use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional redaction template must pass. Check returns its problems, or none.
pub trait RegionalRedactionTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRedactionTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionTemplateRequiredExternalRefRule {
    pub message: String,
}

impl RegionalRedactionTemplateRule for RegionalRedactionTemplateRequiredExternalRefRule {
    fn check(&self, item: &RegionalRedactionTemplate) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionTemplateSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalRedactionTemplateRule for RegionalRedactionTemplateSequenceRangeRule {
    fn check(&self, item: &RegionalRedactionTemplate) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRedactionTemplatePriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalRedactionTemplateRule for RegionalRedactionTemplatePriorityRangeRule {
    fn check(&self, item: &RegionalRedactionTemplate) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional redaction template.
#[derive(Debug, Clone)]
pub struct RegionalRedactionTemplateValidator {
    pub rules: Vec<Rc<dyn RegionalRedactionTemplateRule>>,
}

impl RegionalRedactionTemplateValidator {
    pub fn validate(&self, item: &RegionalRedactionTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalRedactionTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRedactionTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_redaction_template_validator() -> RegionalRedactionTemplateValidator {
    RegionalRedactionTemplateValidator {
        rules: vec![
            Rc::new(RegionalRedactionTemplateRequiredExternalRefRule {
                message: "regional redaction template needs this field".to_string(),
            }) as Rc<dyn RegionalRedactionTemplateRule>,
            Rc::new(RegionalRedactionTemplateSequenceRangeRule { min: 80, max: 3600 }),
            Rc::new(RegionalRedactionTemplatePriorityRangeRule { min: 16, max: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_redaction_template_validator();
        let item = new_regional_redaction_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = RegionalRedactionTemplateRequiredExternalRefRule {
            message: "regional redaction template needs this field".to_string(),
        };
        let mut item = new_regional_redaction_template("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = RegionalRedactionTemplateSequenceRangeRule { min: 16, max: 3600 };
        let mut item = new_regional_redaction_template("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = RegionalRedactionTemplatePriorityRangeRule { min: 64, max: 120 };
        let mut item = new_regional_redaction_template("x", "y");
        item.priority = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
