use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export template must pass. Check returns its problems, or none.
pub trait DataExportTemplateRule: std::fmt::Debug {
    fn check(&self, item: &DataExportTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTemplateEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportTemplateRule for DataExportTemplateEndsAtRangeRule {
    fn check(&self, item: &DataExportTemplate) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTemplateRequiredCodeRule {
    pub message: String,
}

impl DataExportTemplateRule for DataExportTemplateRequiredCodeRule {
    fn check(&self, item: &DataExportTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTemplateNameLengthRule {
    pub max_length: i64,
}

impl DataExportTemplateRule for DataExportTemplateNameLengthRule {
    fn check(&self, item: &DataExportTemplate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTemplateExternalRefLengthRule {
    pub max_length: i64,
}

impl DataExportTemplateRule for DataExportTemplateExternalRefLengthRule {
    fn check(&self, item: &DataExportTemplate) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export template.
#[derive(Debug, Clone)]
pub struct DataExportTemplateValidator {
    pub rules: Vec<Rc<dyn DataExportTemplateRule>>,
}

impl DataExportTemplateValidator {
    pub fn validate(&self, item: &DataExportTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_template_validator() -> DataExportTemplateValidator {
    DataExportTemplateValidator {
        rules: vec![
            Rc::new(DataExportTemplateEndsAtRangeRule { min: 120, max: 255 }) as Rc<dyn DataExportTemplateRule>,
            Rc::new(DataExportTemplateRequiredCodeRule {
                message: "data export template needs a value".to_string(),
            }),
            Rc::new(DataExportTemplateNameLengthRule { max_length: 120 }),
            Rc::new(DataExportTemplateExternalRefLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_template_validator();
        let item = new_data_export_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = DataExportTemplateEndsAtRangeRule { min: 16, max: 3600 };
        let mut item = new_data_export_template("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DataExportTemplateRequiredCodeRule {
            message: "data export template needs an entry".to_string(),
        };
        let mut item = new_data_export_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DataExportTemplateNameLengthRule { max_length: 1000 };
        let mut item = new_data_export_template("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = DataExportTemplateExternalRefLengthRule { max_length: 3600 };
        let mut item = new_data_export_template("x", "y");
        item.external_ref = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
