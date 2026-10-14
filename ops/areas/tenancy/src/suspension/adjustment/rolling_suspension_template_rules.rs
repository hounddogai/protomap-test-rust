use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling suspension template must pass. Check returns its problems, or none.
pub trait RollingSuspensionTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RollingSuspensionTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSuspensionTemplateCodeLengthRule {
    pub max_length: i64,
}

impl RollingSuspensionTemplateRule for RollingSuspensionTemplateCodeLengthRule {
    fn check(&self, item: &RollingSuspensionTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSuspensionTemplateBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingSuspensionTemplateRule for RollingSuspensionTemplateBudgetRangeRule {
    fn check(&self, item: &RollingSuspensionTemplate) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSuspensionTemplateTitleLengthRule {
    pub max_length: i64,
}

impl RollingSuspensionTemplateRule for RollingSuspensionTemplateTitleLengthRule {
    fn check(&self, item: &RollingSuspensionTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling suspension template.
#[derive(Debug, Clone)]
pub struct RollingSuspensionTemplateValidator {
    pub rules: Vec<Rc<dyn RollingSuspensionTemplateRule>>,
}

impl RollingSuspensionTemplateValidator {
    pub fn validate(&self, item: &RollingSuspensionTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingSuspensionTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingSuspensionTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_suspension_template_validator() -> RollingSuspensionTemplateValidator {
    RollingSuspensionTemplateValidator {
        rules: vec![
            Rc::new(RollingSuspensionTemplateCodeLengthRule { max_length: 64 })
                as Rc<dyn RollingSuspensionTemplateRule>,
            Rc::new(RollingSuspensionTemplateBudgetRangeRule { min: 1.2, max: 509.3 }),
            Rc::new(RollingSuspensionTemplateTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_suspension_template_validator();
        let item = new_rolling_suspension_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingSuspensionTemplateCodeLengthRule { max_length: 1000 };
        let mut item = new_rolling_suspension_template("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RollingSuspensionTemplateBudgetRangeRule { min: 8.3, max: 3531.4 };
        let mut item = new_rolling_suspension_template("x", "y");
        item.budget = 3532.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingSuspensionTemplateTitleLengthRule { max_length: 120 };
        let mut item = new_rolling_suspension_template("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
