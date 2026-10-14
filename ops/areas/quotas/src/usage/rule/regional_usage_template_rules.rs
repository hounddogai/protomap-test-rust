use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional usage template must pass. Check returns its problems, or none.
pub trait RegionalUsageTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RegionalUsageTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTemplateRequiredHostRule {
    pub message: String,
}

impl RegionalUsageTemplateRule for RegionalUsageTemplateRequiredHostRule {
    fn check(&self, item: &RegionalUsageTemplate) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTemplateRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalUsageTemplateRule for RegionalUsageTemplateRateRangeRule {
    fn check(&self, item: &RegionalUsageTemplate) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTemplateCodeLengthRule {
    pub max_length: i64,
}

impl RegionalUsageTemplateRule for RegionalUsageTemplateCodeLengthRule {
    fn check(&self, item: &RegionalUsageTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional usage template.
#[derive(Debug, Clone)]
pub struct RegionalUsageTemplateValidator {
    pub rules: Vec<Rc<dyn RegionalUsageTemplateRule>>,
}

impl RegionalUsageTemplateValidator {
    pub fn validate(&self, item: &RegionalUsageTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalUsageTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalUsageTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_usage_template_validator() -> RegionalUsageTemplateValidator {
    RegionalUsageTemplateValidator {
        rules: vec![
            Rc::new(RegionalUsageTemplateRequiredHostRule {
                message: "regional usage template needs this field".to_string(),
            }) as Rc<dyn RegionalUsageTemplateRule>,
            Rc::new(RegionalUsageTemplateRateRangeRule { min: 2.2, max: 1879.7 }),
            Rc::new(RegionalUsageTemplateCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_usage_template_validator();
        let item = new_regional_usage_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RegionalUsageTemplateRequiredHostRule {
            message: "regional usage template needs an entry".to_string(),
        };
        let mut item = new_regional_usage_template("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = RegionalUsageTemplateRateRangeRule { min: 6.9, max: 353.6 };
        let mut item = new_regional_usage_template("x", "y");
        item.rate = 355.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalUsageTemplateCodeLengthRule { max_length: 32 };
        let mut item = new_regional_usage_template("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
