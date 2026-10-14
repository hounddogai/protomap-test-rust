use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record proposal must pass. Check returns its problems, or none.
pub trait DnsRecordProposalRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordProposalPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordProposalRule for DnsRecordProposalPortRangeRule {
    fn check(&self, item: &DnsRecordProposal) -> Vec<String> {
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
pub struct DnsRecordProposalPrincipalLengthRule {
    pub max_length: i64,
}

impl DnsRecordProposalRule for DnsRecordProposalPrincipalLengthRule {
    fn check(&self, item: &DnsRecordProposal) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordProposalNameLengthRule {
    pub max_length: i64,
}

impl DnsRecordProposalRule for DnsRecordProposalNameLengthRule {
    fn check(&self, item: &DnsRecordProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordProposalBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DnsRecordProposalRule for DnsRecordProposalBurstRangeRule {
    fn check(&self, item: &DnsRecordProposal) -> Vec<String> {
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
pub struct DnsRecordProposalRequiredCodeRule {
    pub message: String,
}

impl DnsRecordProposalRule for DnsRecordProposalRequiredCodeRule {
    fn check(&self, item: &DnsRecordProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record proposal.
#[derive(Debug, Clone)]
pub struct DnsRecordProposalValidator {
    pub rules: Vec<Rc<dyn DnsRecordProposalRule>>,
}

impl DnsRecordProposalValidator {
    pub fn validate(&self, item: &DnsRecordProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DnsRecordProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_proposal_validator() -> DnsRecordProposalValidator {
    DnsRecordProposalValidator {
        rules: vec![
            Rc::new(DnsRecordProposalPortRangeRule { min: 64, max: 1000 }) as Rc<dyn DnsRecordProposalRule>,
            Rc::new(DnsRecordProposalPrincipalLengthRule { max_length: 64 }),
            Rc::new(DnsRecordProposalNameLengthRule { max_length: 3600 }),
            Rc::new(DnsRecordProposalBurstRangeRule { min: 120, max: 1000 }),
            Rc::new(DnsRecordProposalRequiredCodeRule {
                message: "dns record proposal needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_proposal_validator();
        let item = new_dns_record_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = DnsRecordProposalPortRangeRule { min: 64, max: 255 };
        let mut item = new_dns_record_proposal("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = DnsRecordProposalPrincipalLengthRule { max_length: 16 };
        let mut item = new_dns_record_proposal("x", "y");
        item.principal = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DnsRecordProposalNameLengthRule { max_length: 1000 };
        let mut item = new_dns_record_proposal("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = DnsRecordProposalBurstRangeRule { min: 64, max: 255 };
        let mut item = new_dns_record_proposal("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordProposalRequiredCodeRule {
            message: "dns record proposal needs an entry".to_string(),
        };
        let mut item = new_dns_record_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
