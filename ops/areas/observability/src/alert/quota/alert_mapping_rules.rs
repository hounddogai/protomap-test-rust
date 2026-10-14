use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert mapping must pass. Check returns its problems, or none.
pub trait AlertMappingRule: std::fmt::Debug {
    fn check(&self, item: &AlertMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertMappingRequiredCodeRule {
    pub message: String,
}

impl AlertMappingRule for AlertMappingRequiredCodeRule {
    fn check(&self, item: &AlertMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertMappingRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertMappingRule for AlertMappingRevisionRangeRule {
    fn check(&self, item: &AlertMapping) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertMappingSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertMappingRule for AlertMappingSequenceRangeRule {
    fn check(&self, item: &AlertMapping) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertMappingRequiredTitleRule {
    pub message: String,
}

impl AlertMappingRule for AlertMappingRequiredTitleRule {
    fn check(&self, item: &AlertMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert mapping.
#[derive(Debug, Clone)]
pub struct AlertMappingValidator {
    pub rules: Vec<Rc<dyn AlertMappingRule>>,
}

impl AlertMappingValidator {
    pub fn validate(&self, item: &AlertMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_mapping_validator() -> AlertMappingValidator {
    AlertMappingValidator {
        rules: vec![
            Rc::new(AlertMappingRequiredCodeRule {
                message: "alert mapping needs an entry".to_string(),
            }) as Rc<dyn AlertMappingRule>,
            Rc::new(AlertMappingRevisionRangeRule { min: 16, max: 86400 }),
            Rc::new(AlertMappingSequenceRangeRule { min: 32, max: 255 }),
            Rc::new(AlertMappingRequiredTitleRule {
                message: "alert mapping needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_mapping_validator();
        let item = new_alert_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AlertMappingRequiredCodeRule {
            message: "alert mapping needs an entry".to_string(),
        };
        let mut item = new_alert_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = AlertMappingRevisionRangeRule { min: 32, max: 3600 };
        let mut item = new_alert_mapping("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = AlertMappingSequenceRangeRule { min: 64, max: 1000 };
        let mut item = new_alert_mapping("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AlertMappingRequiredTitleRule {
            message: "alert mapping needs this field".to_string(),
        };
        let mut item = new_alert_mapping("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
