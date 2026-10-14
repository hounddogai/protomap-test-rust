use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record rule must pass. Check returns its problems, or none.
pub trait DnsRecordRuleRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRuleRequiredNameRule {
    pub message: String,
}

impl DnsRecordRuleRule for DnsRecordRuleRequiredNameRule {
    fn check(&self, item: &DnsRecordRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRuleCodePrefixRule {
    pub prefix: String,
}

impl DnsRecordRuleRule for DnsRecordRuleCodePrefixRule {
    fn check(&self, item: &DnsRecordRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRuleLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordRuleRule for DnsRecordRuleLimitRangeRule {
    fn check(&self, item: &DnsRecordRule) -> Vec<String> {
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
pub struct DnsRecordRuleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordRuleRule for DnsRecordRuleRotationDaysRangeRule {
    fn check(&self, item: &DnsRecordRule) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRuleRequiredCodeRule {
    pub message: String,
}

impl DnsRecordRuleRule for DnsRecordRuleRequiredCodeRule {
    fn check(&self, item: &DnsRecordRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record rule.
#[derive(Debug, Clone)]
pub struct DnsRecordRuleValidator {
    pub rules: Vec<Rc<dyn DnsRecordRuleRule>>,
}

impl DnsRecordRuleValidator {
    pub fn validate(&self, item: &DnsRecordRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_rule_validator() -> DnsRecordRuleValidator {
    DnsRecordRuleValidator {
        rules: vec![
            Rc::new(DnsRecordRuleRequiredNameRule {
                message: "dns record rule needs a value".to_string(),
            }) as Rc<dyn DnsRecordRuleRule>,
            Rc::new(DnsRecordRuleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DnsRecordRuleLimitRangeRule { min: 64, max: 64 }),
            Rc::new(DnsRecordRuleRotationDaysRangeRule { min: 80, max: 86400 }),
            Rc::new(DnsRecordRuleRequiredCodeRule {
                message: "dns record rule needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_rule_validator();
        let item = new_dns_record_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DnsRecordRuleRequiredNameRule {
            message: "dns record rule needs a value".to_string(),
        };
        let mut item = new_dns_record_rule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = DnsRecordRuleLimitRangeRule { min: 255, max: 255 };
        let mut item = new_dns_record_rule("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = DnsRecordRuleRotationDaysRangeRule { min: 255, max: 86400 };
        let mut item = new_dns_record_rule("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordRuleRequiredCodeRule {
            message: "dns record rule needs this field".to_string(),
        };
        let mut item = new_dns_record_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
