use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export proposal must pass. Check returns its problems, or none.
pub trait DataExportProposalRule: std::fmt::Debug {
    fn check(&self, item: &DataExportProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProposalCodeLengthRule {
    pub max_length: i64,
}

impl DataExportProposalRule for DataExportProposalCodeLengthRule {
    fn check(&self, item: &DataExportProposal) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProposalRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportProposalRule for DataExportProposalRatioRangeRule {
    fn check(&self, item: &DataExportProposal) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProposalNameLengthRule {
    pub max_length: i64,
}

impl DataExportProposalRule for DataExportProposalNameLengthRule {
    fn check(&self, item: &DataExportProposal) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProposalRequiredCodeRule {
    pub message: String,
}

impl DataExportProposalRule for DataExportProposalRequiredCodeRule {
    fn check(&self, item: &DataExportProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportProposalRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportProposalRule for DataExportProposalRetriesRangeRule {
    fn check(&self, item: &DataExportProposal) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export proposal.
#[derive(Debug, Clone)]
pub struct DataExportProposalValidator {
    pub rules: Vec<Rc<dyn DataExportProposalRule>>,
}

impl DataExportProposalValidator {
    pub fn validate(&self, item: &DataExportProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DataExportProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_proposal_validator() -> DataExportProposalValidator {
    DataExportProposalValidator {
        rules: vec![
            Rc::new(DataExportProposalCodeLengthRule { max_length: 16 }) as Rc<dyn DataExportProposalRule>,
            Rc::new(DataExportProposalRatioRangeRule { min: 5.8, max: 2989.8 }),
            Rc::new(DataExportProposalNameLengthRule { max_length: 80 }),
            Rc::new(DataExportProposalRequiredCodeRule {
                message: "data export proposal needs a value".to_string(),
            }),
            Rc::new(DataExportProposalRetriesRangeRule { min: 64, max: 255 }),
        ],
    }
}
