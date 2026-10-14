use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global rollback draft calculation.
pub trait GlobalRollbackDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalRollbackDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRollbackDraftCeilingOffsetAdjuster {
    pub weight: f64,
}

impl GlobalRollbackDraftModifier for GlobalRollbackDraftCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalRollbackDraft, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalRollbackDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalRollbackDraftModifier for FlaggedGlobalRollbackDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalRollbackDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalRollbackDraftAdjuster {
    pub percent: f64,
}

impl GlobalRollbackDraftModifier for PercentGlobalRollbackDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalRollbackDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global rollback draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalRollbackDraftEngine {
    pub steps: Vec<Rc<dyn GlobalRollbackDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalRollbackDraftEngine {
    pub fn compute(&mut self, item: &GlobalRollbackDraft) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalRollbackDraftModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalRollbackDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_rollback_draft_engine(flags: &flags::Registry) -> GlobalRollbackDraftEngine {
    GlobalRollbackDraftEngine {
        steps: vec![
            Rc::new(GlobalRollbackDraftCeilingOffsetAdjuster { weight: 0.92 }) as Rc<dyn GlobalRollbackDraftModifier>,
            Rc::new(FlaggedGlobalRollbackDraftAdjuster {
                flags: flags.clone(),
                flag: "global-rollback-draft-weighted".to_string(),
                percent: 5.5,
            }),
            Rc::new(PercentGlobalRollbackDraftAdjuster { percent: 19.1 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 96754.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_rollback_draft_engine(&flags::new_registry());
        let item = new_global_rollback_draft("code-680", "title-857");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
