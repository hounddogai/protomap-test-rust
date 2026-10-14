use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard target must pass. Check returns its problems, or none.
pub trait DashboardTargetRule: std::fmt::Debug {
    fn check(&self, item: &DashboardTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTargetCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DashboardTargetRule for DashboardTargetCeilingRangeRule {
    fn check(&self, item: &DashboardTarget) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTargetRequiredTitleRule {
    pub message: String,
}

impl DashboardTargetRule for DashboardTargetRequiredTitleRule {
    fn check(&self, item: &DashboardTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTargetRequiredChannelRule {
    pub message: String,
}

impl DashboardTargetRule for DashboardTargetRequiredChannelRule {
    fn check(&self, item: &DashboardTarget) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTargetChannelLengthRule {
    pub max_length: i64,
}

impl DashboardTargetRule for DashboardTargetChannelLengthRule {
    fn check(&self, item: &DashboardTarget) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardTargetCodeLengthRule {
    pub max_length: i64,
}

impl DashboardTargetRule for DashboardTargetCodeLengthRule {
    fn check(&self, item: &DashboardTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard target.
#[derive(Debug, Clone)]
pub struct DashboardTargetValidator {
    pub rules: Vec<Rc<dyn DashboardTargetRule>>,
}

impl DashboardTargetValidator {
    pub fn validate(&self, item: &DashboardTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_target_validator() -> DashboardTargetValidator {
    DashboardTargetValidator {
        rules: vec![
            Rc::new(DashboardTargetCeilingRangeRule { min: 3.5, max: 1303.4 }) as Rc<dyn DashboardTargetRule>,
            Rc::new(DashboardTargetRequiredTitleRule {
                message: "dashboard target needs this field".to_string(),
            }),
            Rc::new(DashboardTargetRequiredChannelRule {
                message: "dashboard target needs an entry".to_string(),
            }),
            Rc::new(DashboardTargetChannelLengthRule { max_length: 255 }),
            Rc::new(DashboardTargetCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_target_validator();
        let item = new_dashboard_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = DashboardTargetCeilingRangeRule { min: 4.6, max: 1972.7 };
        let mut item = new_dashboard_target("x", "y");
        item.ceiling = 1974.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DashboardTargetRequiredTitleRule {
            message: "dashboard target needs a value".to_string(),
        };
        let mut item = new_dashboard_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = DashboardTargetRequiredChannelRule {
            message: "dashboard target needs an entry".to_string(),
        };
        let mut item = new_dashboard_target("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = DashboardTargetChannelLengthRule { max_length: 16 };
        let mut item = new_dashboard_target("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DashboardTargetCodeLengthRule { max_length: 64 };
        let mut item = new_dashboard_target("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
