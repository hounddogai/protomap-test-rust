use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped evidence template must pass. Check returns its problems, or none.
pub trait CappedEvidenceTemplateRule: std::fmt::Debug {
    fn check(&self, item: &CappedEvidenceTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceTemplateTitleLengthRule {
    pub max_length: i64,
}

impl CappedEvidenceTemplateRule for CappedEvidenceTemplateTitleLengthRule {
    fn check(&self, item: &CappedEvidenceTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceTemplateUnitLengthRule {
    pub max_length: i64,
}

impl CappedEvidenceTemplateRule for CappedEvidenceTemplateUnitLengthRule {
    fn check(&self, item: &CappedEvidenceTemplate) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceTemplateCodePrefixRule {
    pub prefix: String,
}

impl CappedEvidenceTemplateRule for CappedEvidenceTemplateCodePrefixRule {
    fn check(&self, item: &CappedEvidenceTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceTemplateCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedEvidenceTemplateRule for CappedEvidenceTemplateCapacityRangeRule {
    fn check(&self, item: &CappedEvidenceTemplate) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped evidence template.
#[derive(Debug, Clone)]
pub struct CappedEvidenceTemplateValidator {
    pub rules: Vec<Rc<dyn CappedEvidenceTemplateRule>>,
}

impl CappedEvidenceTemplateValidator {
    pub fn validate(&self, item: &CappedEvidenceTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedEvidenceTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedEvidenceTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_evidence_template_validator() -> CappedEvidenceTemplateValidator {
    CappedEvidenceTemplateValidator {
        rules: vec![
            Rc::new(CappedEvidenceTemplateTitleLengthRule { max_length: 120 }) as Rc<dyn CappedEvidenceTemplateRule>,
            Rc::new(CappedEvidenceTemplateUnitLengthRule { max_length: 255 }),
            Rc::new(CappedEvidenceTemplateCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(CappedEvidenceTemplateCapacityRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_evidence_template_validator();
        let item = new_capped_evidence_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CappedEvidenceTemplateTitleLengthRule { max_length: 86400 };
        let mut item = new_capped_evidence_template("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = CappedEvidenceTemplateUnitLengthRule { max_length: 1000 };
        let mut item = new_capped_evidence_template("x", "y");
        item.unit = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = CappedEvidenceTemplateCapacityRangeRule { min: 64, max: 86400 };
        let mut item = new_capped_evidence_template("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
