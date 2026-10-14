use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global dashboard notice must pass. Check returns its problems, or none.
pub trait GlobalDashboardNoticeRule: std::fmt::Debug {
    fn check(&self, item: &GlobalDashboardNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDashboardNoticeEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalDashboardNoticeRule for GlobalDashboardNoticeEndsAtRangeRule {
    fn check(&self, item: &GlobalDashboardNotice) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDashboardNoticeRequiredLabelRule {
    pub message: String,
}

impl GlobalDashboardNoticeRule for GlobalDashboardNoticeRequiredLabelRule {
    fn check(&self, item: &GlobalDashboardNotice) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDashboardNoticeCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalDashboardNoticeRule for GlobalDashboardNoticeCeilingRangeRule {
    fn check(&self, item: &GlobalDashboardNotice) -> Vec<String> {
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

/// Runs every rule of a global dashboard notice.
#[derive(Debug, Clone)]
pub struct GlobalDashboardNoticeValidator {
    pub rules: Vec<Rc<dyn GlobalDashboardNoticeRule>>,
}

impl GlobalDashboardNoticeValidator {
    pub fn validate(&self, item: &GlobalDashboardNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalDashboardNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalDashboardNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_dashboard_notice_validator() -> GlobalDashboardNoticeValidator {
    GlobalDashboardNoticeValidator {
        rules: vec![
            Rc::new(GlobalDashboardNoticeEndsAtRangeRule { min: 120, max: 86400 }) as Rc<dyn GlobalDashboardNoticeRule>,
            Rc::new(GlobalDashboardNoticeRequiredLabelRule {
                message: "global dashboard notice needs a value".to_string(),
            }),
            Rc::new(GlobalDashboardNoticeCeilingRangeRule { min: 3.8, max: 1004.0 }),
        ],
    }
}
