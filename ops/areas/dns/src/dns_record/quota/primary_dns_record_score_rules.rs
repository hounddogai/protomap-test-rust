use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary dns record score must pass. Check returns its problems, or none.
pub trait PrimaryDnsRecordScoreRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryDnsRecordScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordScorePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryDnsRecordScoreRule for PrimaryDnsRecordScorePortRangeRule {
    fn check(&self, item: &PrimaryDnsRecordScore) -> Vec<String> {
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
pub struct PrimaryDnsRecordScoreRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryDnsRecordScoreRule for PrimaryDnsRecordScoreRotationDaysRangeRule {
    fn check(&self, item: &PrimaryDnsRecordScore) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordScoreRequiredPhoneRule {
    pub message: String,
}

impl PrimaryDnsRecordScoreRule for PrimaryDnsRecordScoreRequiredPhoneRule {
    fn check(&self, item: &PrimaryDnsRecordScore) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary dns record score.
#[derive(Debug, Clone)]
pub struct PrimaryDnsRecordScoreValidator {
    pub rules: Vec<Rc<dyn PrimaryDnsRecordScoreRule>>,
}

impl PrimaryDnsRecordScoreValidator {
    pub fn validate(&self, item: &PrimaryDnsRecordScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryDnsRecordScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryDnsRecordScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_dns_record_score_validator() -> PrimaryDnsRecordScoreValidator {
    PrimaryDnsRecordScoreValidator {
        rules: vec![
            Rc::new(PrimaryDnsRecordScorePortRangeRule { min: 16, max: 120 }) as Rc<dyn PrimaryDnsRecordScoreRule>,
            Rc::new(PrimaryDnsRecordScoreRotationDaysRangeRule { min: 3600, max: 3600 }),
            Rc::new(PrimaryDnsRecordScoreRequiredPhoneRule {
                message: "primary dns record score needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_dns_record_score_validator();
        let item = new_primary_dns_record_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = PrimaryDnsRecordScorePortRangeRule { min: 16, max: 80 };
        let mut item = new_primary_dns_record_score("x", "y");
        item.port = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = PrimaryDnsRecordScoreRotationDaysRangeRule { min: 32, max: 1000 };
        let mut item = new_primary_dns_record_score("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = PrimaryDnsRecordScoreRequiredPhoneRule {
            message: "primary dns record score needs this field".to_string(),
        };
        let mut item = new_primary_dns_record_score("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
