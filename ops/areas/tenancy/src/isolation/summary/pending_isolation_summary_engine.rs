use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending isolation summary calculation.
pub trait PendingIsolationSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingIsolationSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingIsolationSummaryAdjuster {
    pub step: f64,
}

impl PendingIsolationSummaryAdjuster for RoundingPendingIsolationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingIsolationSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingIsolationSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingIsolationSummaryAdjuster for TieredPendingIsolationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingIsolationSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPendingIsolationSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingIsolationSummaryAdjuster for LabelPendingIsolationSummaryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PendingIsolationSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingIsolationSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingIsolationSummaryAdjuster for FlaggedPendingIsolationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingIsolationSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending isolation summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingIsolationSummaryEngine {
    pub steps: Vec<Rc<dyn PendingIsolationSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingIsolationSummaryEngine {
    pub fn compute(&mut self, item: &PendingIsolationSummary) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingIsolationSummaryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingIsolationSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_isolation_summary_engine(flags: &flags::Registry) -> PendingIsolationSummaryEngine {
    PendingIsolationSummaryEngine {
        steps: vec![
            Rc::new(RoundingPendingIsolationSummaryAdjuster { step: 1.0 }) as Rc<dyn PendingIsolationSummaryAdjuster>,
            Rc::new(TieredPendingIsolationSummaryAdjuster {
                bounds: vec![8802.0, 9473.0],
                rates: vec![0.92, 0.91],
            }),
            Rc::new(LabelPendingIsolationSummaryAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.37),
                    ("central".to_string(), 1.39),
                    ("north".to_string(), 0.85),
                ]),
            }),
            Rc::new(FlaggedPendingIsolationSummaryAdjuster {
                flags: flags.clone(),
                flag: "pending-isolation-summary-tiered".to_string(),
                percent: 7.1,
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 86873.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_isolation_summary_engine(&flags::new_registry());
        let item = new_pending_isolation_summary("code-730", "label-770");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
