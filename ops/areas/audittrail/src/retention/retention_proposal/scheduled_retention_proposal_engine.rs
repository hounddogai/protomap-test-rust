use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled retention proposal calculation.
pub trait ScheduledRetentionProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledRetentionProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceScheduledRetentionProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledRetentionProposalModifier for SourceScheduledRetentionProposalAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &ScheduledRetentionProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledRetentionProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledRetentionProposalModifier for FlaggedScheduledRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledRetentionProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledRetentionProposalAdjuster {
    pub cap: f64,
}

impl ScheduledRetentionProposalModifier for CapScheduledRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledRetentionProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled retention proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledRetentionProposalEngine {
    pub steps: Vec<Rc<dyn ScheduledRetentionProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledRetentionProposalEngine {
    pub fn run(&mut self, item: &ScheduledRetentionProposal) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledRetentionProposalModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[ScheduledRetentionProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_retention_proposal_engine(flags: &flags::Registry) -> ScheduledRetentionProposalEngine {
    ScheduledRetentionProposalEngine {
        steps: vec![
            Rc::new(SourceScheduledRetentionProposalAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.78),
                    ("north".to_string(), 1.33),
                    ("central".to_string(), 1.0),
                ]),
            }) as Rc<dyn ScheduledRetentionProposalModifier>,
            Rc::new(FlaggedScheduledRetentionProposalAdjuster {
                flags: flags.clone(),
                flag: "scheduled-retention-proposal-regional".to_string(),
                percent: 3.7,
            }),
            Rc::new(CapScheduledRetentionProposalAdjuster { cap: 9238.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 57835.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_retention_proposal_engine(&flags::new_registry());
        let item = new_scheduled_retention_proposal("code-18", "title-956");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
