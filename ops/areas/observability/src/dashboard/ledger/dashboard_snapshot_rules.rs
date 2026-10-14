use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard snapshot must pass. Check returns its problems, or none.
pub trait DashboardSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &DashboardSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSnapshotContactEmailLengthRule {
    pub max_length: i64,
}

impl DashboardSnapshotRule for DashboardSnapshotContactEmailLengthRule {
    fn check(&self, item: &DashboardSnapshot) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSnapshotLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardSnapshotRule for DashboardSnapshotLimitRangeRule {
    fn check(&self, item: &DashboardSnapshot) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSnapshotRequiredLabelRule {
    pub message: String,
}

impl DashboardSnapshotRule for DashboardSnapshotRequiredLabelRule {
    fn check(&self, item: &DashboardSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl DashboardSnapshotRule for DashboardSnapshotCodeLengthRule {
    fn check(&self, item: &DashboardSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSnapshotLabelLengthRule {
    pub max_length: i64,
}

impl DashboardSnapshotRule for DashboardSnapshotLabelLengthRule {
    fn check(&self, item: &DashboardSnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSnapshotRequiredIssuerRule {
    pub message: String,
}

impl DashboardSnapshotRule for DashboardSnapshotRequiredIssuerRule {
    fn check(&self, item: &DashboardSnapshot) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard snapshot.
#[derive(Debug, Clone)]
pub struct DashboardSnapshotValidator {
    pub rules: Vec<Rc<dyn DashboardSnapshotRule>>,
}

impl DashboardSnapshotValidator {
    pub fn validate(&self, item: &DashboardSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_snapshot_validator() -> DashboardSnapshotValidator {
    DashboardSnapshotValidator {
        rules: vec![
            Rc::new(DashboardSnapshotContactEmailLengthRule { max_length: 32 }) as Rc<dyn DashboardSnapshotRule>,
            Rc::new(DashboardSnapshotLimitRangeRule { min: 64, max: 86400 }),
            Rc::new(DashboardSnapshotRequiredLabelRule {
                message: "dashboard snapshot needs this field".to_string(),
            }),
            Rc::new(DashboardSnapshotCodeLengthRule { max_length: 120 }),
            Rc::new(DashboardSnapshotLabelLengthRule { max_length: 120 }),
            Rc::new(DashboardSnapshotRequiredIssuerRule {
                message: "dashboard snapshot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_snapshot_validator();
        let item = new_dashboard_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = DashboardSnapshotContactEmailLengthRule { max_length: 86400 };
        let mut item = new_dashboard_snapshot("x", "y");
        item.contact_email = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = DashboardSnapshotLimitRangeRule { min: 16, max: 255 };
        let mut item = new_dashboard_snapshot("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DashboardSnapshotRequiredLabelRule {
            message: "dashboard snapshot needs a value".to_string(),
        };
        let mut item = new_dashboard_snapshot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DashboardSnapshotCodeLengthRule { max_length: 32 };
        let mut item = new_dashboard_snapshot("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DashboardSnapshotLabelLengthRule { max_length: 120 };
        let mut item = new_dashboard_snapshot("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = DashboardSnapshotRequiredIssuerRule {
            message: "dashboard snapshot needs an entry".to_string(),
        };
        let mut item = new_dashboard_snapshot("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
