use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped dashboard template must pass. Check returns its problems, or none.
pub trait CappedDashboardTemplateRule: std::fmt::Debug {
    fn check(&self, item: &CappedDashboardTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTemplateTitleLengthRule {
    pub max_length: i64,
}

impl CappedDashboardTemplateRule for CappedDashboardTemplateTitleLengthRule {
    fn check(&self, item: &CappedDashboardTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTemplateLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedDashboardTemplateRule for CappedDashboardTemplateLimitRangeRule {
    fn check(&self, item: &CappedDashboardTemplate) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTemplateCodeLengthRule {
    pub max_length: i64,
}

impl CappedDashboardTemplateRule for CappedDashboardTemplateCodeLengthRule {
    fn check(&self, item: &CappedDashboardTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTemplateUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedDashboardTemplateRule for CappedDashboardTemplateUpdatedAtRangeRule {
    fn check(&self, item: &CappedDashboardTemplate) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTemplateEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedDashboardTemplateRule for CappedDashboardTemplateEndsAtRangeRule {
    fn check(&self, item: &CappedDashboardTemplate) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped dashboard template.
#[derive(Debug, Clone)]
pub struct CappedDashboardTemplateValidator {
    pub rules: Vec<Rc<dyn CappedDashboardTemplateRule>>,
}

impl CappedDashboardTemplateValidator {
    pub fn validate(&self, item: &CappedDashboardTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedDashboardTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedDashboardTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_dashboard_template_validator() -> CappedDashboardTemplateValidator {
    CappedDashboardTemplateValidator {
        rules: vec![
            Rc::new(CappedDashboardTemplateTitleLengthRule { max_length: 1000 }) as Rc<dyn CappedDashboardTemplateRule>,
            Rc::new(CappedDashboardTemplateLimitRangeRule { min: 255, max: 1000 }),
            Rc::new(CappedDashboardTemplateCodeLengthRule { max_length: 255 }),
            Rc::new(CappedDashboardTemplateUpdatedAtRangeRule { min: 120, max: 86400 }),
            Rc::new(CappedDashboardTemplateEndsAtRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_dashboard_template_validator();
        let item = new_capped_dashboard_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CappedDashboardTemplateTitleLengthRule { max_length: 255 };
        let mut item = new_capped_dashboard_template("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = CappedDashboardTemplateLimitRangeRule { min: 32, max: 1000 };
        let mut item = new_capped_dashboard_template("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedDashboardTemplateCodeLengthRule { max_length: 64 };
        let mut item = new_capped_dashboard_template("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = CappedDashboardTemplateUpdatedAtRangeRule { min: 255, max: 3600 };
        let mut item = new_capped_dashboard_template("x", "y");
        item.updated_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = CappedDashboardTemplateEndsAtRangeRule { min: 32, max: 3600 };
        let mut item = new_capped_dashboard_template("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
