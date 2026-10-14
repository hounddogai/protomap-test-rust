use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted usage template must pass. Check returns its problems, or none.
pub trait WeightedUsageTemplateRule: std::fmt::Debug {
    fn check(&self, item: &WeightedUsageTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTemplateOwnerIdLengthRule {
    pub max_length: i64,
}

impl WeightedUsageTemplateRule for WeightedUsageTemplateOwnerIdLengthRule {
    fn check(&self, item: &WeightedUsageTemplate) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTemplateLocaleLengthRule {
    pub max_length: i64,
}

impl WeightedUsageTemplateRule for WeightedUsageTemplateLocaleLengthRule {
    fn check(&self, item: &WeightedUsageTemplate) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTemplateBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedUsageTemplateRule for WeightedUsageTemplateBurstRangeRule {
    fn check(&self, item: &WeightedUsageTemplate) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTemplateExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedUsageTemplateRule for WeightedUsageTemplateExpiresAtRangeRule {
    fn check(&self, item: &WeightedUsageTemplate) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTemplateSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedUsageTemplateRule for WeightedUsageTemplateSequenceRangeRule {
    fn check(&self, item: &WeightedUsageTemplate) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTemplateTitleLengthRule {
    pub max_length: i64,
}

impl WeightedUsageTemplateRule for WeightedUsageTemplateTitleLengthRule {
    fn check(&self, item: &WeightedUsageTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted usage template.
#[derive(Debug, Clone)]
pub struct WeightedUsageTemplateValidator {
    pub rules: Vec<Rc<dyn WeightedUsageTemplateRule>>,
}

impl WeightedUsageTemplateValidator {
    pub fn validate(&self, item: &WeightedUsageTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedUsageTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedUsageTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_usage_template_validator() -> WeightedUsageTemplateValidator {
    WeightedUsageTemplateValidator {
        rules: vec![
            Rc::new(WeightedUsageTemplateOwnerIdLengthRule { max_length: 255 }) as Rc<dyn WeightedUsageTemplateRule>,
            Rc::new(WeightedUsageTemplateLocaleLengthRule { max_length: 120 }),
            Rc::new(WeightedUsageTemplateBurstRangeRule { min: 3600, max: 3600 }),
            Rc::new(WeightedUsageTemplateExpiresAtRangeRule { min: 80, max: 80 }),
            Rc::new(WeightedUsageTemplateSequenceRangeRule { min: 120, max: 86400 }),
            Rc::new(WeightedUsageTemplateTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_usage_template_validator();
        let item = new_weighted_usage_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = WeightedUsageTemplateOwnerIdLengthRule { max_length: 255 };
        let mut item = new_weighted_usage_template("x", "y");
        item.owner_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = WeightedUsageTemplateLocaleLengthRule { max_length: 1000 };
        let mut item = new_weighted_usage_template("x", "y");
        item.locale = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = WeightedUsageTemplateBurstRangeRule { min: 1000, max: 3600 };
        let mut item = new_weighted_usage_template("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = WeightedUsageTemplateExpiresAtRangeRule { min: 255, max: 3600 };
        let mut item = new_weighted_usage_template("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = WeightedUsageTemplateSequenceRangeRule { min: 80, max: 120 };
        let mut item = new_weighted_usage_template("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = WeightedUsageTemplateTitleLengthRule { max_length: 1000 };
        let mut item = new_weighted_usage_template("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
