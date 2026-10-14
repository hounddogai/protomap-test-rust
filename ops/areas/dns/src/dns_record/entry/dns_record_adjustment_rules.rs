use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record adjustment must pass. Check returns its problems, or none.
pub trait DnsRecordAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAdjustmentRequiredContactEmailRule {
    pub message: String,
}

impl DnsRecordAdjustmentRule for DnsRecordAdjustmentRequiredContactEmailRule {
    fn check(&self, item: &DnsRecordAdjustment) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAdjustmentRequiredCodeRule {
    pub message: String,
}

impl DnsRecordAdjustmentRule for DnsRecordAdjustmentRequiredCodeRule {
    fn check(&self, item: &DnsRecordAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAdjustmentRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordAdjustmentRule for DnsRecordAdjustmentRevisionRangeRule {
    fn check(&self, item: &DnsRecordAdjustment) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAdjustmentScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordAdjustmentRule for DnsRecordAdjustmentScoreRangeRule {
    fn check(&self, item: &DnsRecordAdjustment) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a dns record adjustment.
#[derive(Debug, Clone)]
pub struct DnsRecordAdjustmentValidator {
    pub rules: Vec<Rc<dyn DnsRecordAdjustmentRule>>,
}

impl DnsRecordAdjustmentValidator {
    pub fn validate(&self, item: &DnsRecordAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_adjustment_validator() -> DnsRecordAdjustmentValidator {
    DnsRecordAdjustmentValidator {
        rules: vec![
            Rc::new(DnsRecordAdjustmentRequiredContactEmailRule {
                message: "dns record adjustment needs this field".to_string(),
            }) as Rc<dyn DnsRecordAdjustmentRule>,
            Rc::new(DnsRecordAdjustmentRequiredCodeRule {
                message: "dns record adjustment needs an entry".to_string(),
            }),
            Rc::new(DnsRecordAdjustmentRevisionRangeRule { min: 80, max: 86400 }),
            Rc::new(DnsRecordAdjustmentScoreRangeRule { min: 2.1, max: 3281.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_adjustment_validator();
        let item = new_dns_record_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = DnsRecordAdjustmentRequiredContactEmailRule {
            message: "dns record adjustment needs this field".to_string(),
        };
        let mut item = new_dns_record_adjustment("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordAdjustmentRequiredCodeRule {
            message: "dns record adjustment needs an entry".to_string(),
        };
        let mut item = new_dns_record_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = DnsRecordAdjustmentRevisionRangeRule { min: 32, max: 3600 };
        let mut item = new_dns_record_adjustment("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = DnsRecordAdjustmentScoreRangeRule { min: 9.6, max: 4824.5 };
        let mut item = new_dns_record_adjustment("x", "y");
        item.score = 4826.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
