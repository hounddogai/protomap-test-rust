use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard assignment must pass. Check returns its problems, or none.
pub trait DashboardAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &DashboardAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAssignmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardAssignmentRule for DashboardAssignmentPortRangeRule {
    fn check(&self, item: &DashboardAssignment) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAssignmentRequiredNameRule {
    pub message: String,
}

impl DashboardAssignmentRule for DashboardAssignmentRequiredNameRule {
    fn check(&self, item: &DashboardAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAssignmentCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardAssignmentRule for DashboardAssignmentCapacityRangeRule {
    fn check(&self, item: &DashboardAssignment) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAssignmentVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardAssignmentRule for DashboardAssignmentVersionRangeRule {
    fn check(&self, item: &DashboardAssignment) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a dashboard assignment.
#[derive(Debug, Clone)]
pub struct DashboardAssignmentValidator {
    pub rules: Vec<Rc<dyn DashboardAssignmentRule>>,
}

impl DashboardAssignmentValidator {
    pub fn validate(&self, item: &DashboardAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_assignment_validator() -> DashboardAssignmentValidator {
    DashboardAssignmentValidator {
        rules: vec![
            Rc::new(DashboardAssignmentPortRangeRule { min: 16, max: 32 }) as Rc<dyn DashboardAssignmentRule>,
            Rc::new(DashboardAssignmentRequiredNameRule {
                message: "dashboard assignment needs a value".to_string(),
            }),
            Rc::new(DashboardAssignmentCapacityRangeRule { min: 16, max: 32 }),
            Rc::new(DashboardAssignmentVersionRangeRule { min: 32, max: 255 }),
        ],
    }
}
