use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending budget burn record calculation.
pub trait PendingBudgetBurnRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingBudgetBurnRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBudgetBurnRecordWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl PendingBudgetBurnRecordAdjuster for PendingBudgetBurnRecordWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &PendingBudgetBurnRecord, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingBudgetBurnRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingBudgetBurnRecordAdjuster for FlaggedPendingBudgetBurnRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingBudgetBurnRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingBudgetBurnRecordAdjuster {
    pub step: f64,
}

impl PendingBudgetBurnRecordAdjuster for RoundingPendingBudgetBurnRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingBudgetBurnRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingBudgetBurnRecordAdjuster {
    pub percent: f64,
}

impl PendingBudgetBurnRecordAdjuster for PercentPendingBudgetBurnRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingBudgetBurnRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingBudgetBurnRecordAdjuster {
    pub cap: f64,
}

impl PendingBudgetBurnRecordAdjuster for CapPendingBudgetBurnRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingBudgetBurnRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePendingBudgetBurnRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingBudgetBurnRecordAdjuster for SourcePendingBudgetBurnRecordAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PendingBudgetBurnRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending budget burn record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingBudgetBurnRecordEngine {
    pub steps: Vec<Rc<dyn PendingBudgetBurnRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingBudgetBurnRecordEngine {
    pub fn run(&mut self, item: &PendingBudgetBurnRecord) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingBudgetBurnRecordAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingBudgetBurnRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_budget_burn_record_engine(flags: &flags::Registry) -> PendingBudgetBurnRecordEngine {
    PendingBudgetBurnRecordEngine {
        steps: vec![
            Rc::new(PendingBudgetBurnRecordWindowDaysOffsetAdjuster { weight: 1.59 })
                as Rc<dyn PendingBudgetBurnRecordAdjuster>,
            Rc::new(FlaggedPendingBudgetBurnRecordAdjuster {
                flags: flags.clone(),
                flag: "pending-budget-burn-record-adaptive".to_string(),
                percent: 9.9,
            }),
            Rc::new(RoundingPendingBudgetBurnRecordAdjuster { step: 0.05 }),
            Rc::new(PercentPendingBudgetBurnRecordAdjuster { percent: -13.7 }),
            Rc::new(CapPendingBudgetBurnRecordAdjuster { cap: 30502.0 }),
            Rc::new(SourcePendingBudgetBurnRecordAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.81)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 29690.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_budget_burn_record_engine(&flags::new_registry());
        let item = new_pending_budget_burn_record("code-949", "name-84");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
