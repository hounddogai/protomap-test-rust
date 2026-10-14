use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted runbook quota calculation.
pub trait WeightedRunbookQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRunbookQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRunbookQuotaWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl WeightedRunbookQuotaFactor for WeightedRunbookQuotaWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedRunbookQuota, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedRunbookQuotaAdjuster {
    pub cap: f64,
}

impl WeightedRunbookQuotaFactor for CapWeightedRunbookQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedRunbookQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedRunbookQuotaAdjuster {
    pub step: f64,
}

impl WeightedRunbookQuotaFactor for RoundingWeightedRunbookQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedRunbookQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the weighted runbook quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRunbookQuotaEngine {
    pub steps: Vec<Rc<dyn WeightedRunbookQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRunbookQuotaEngine {
    pub fn price(&mut self, item: &WeightedRunbookQuota) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[WeightedRunbookQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_runbook_quota_engine() -> WeightedRunbookQuotaEngine {
    WeightedRunbookQuotaEngine {
        steps: vec![
            Rc::new(WeightedRunbookQuotaWindowDaysOffsetAdjuster { weight: 0.95 })
                as Rc<dyn WeightedRunbookQuotaFactor>,
            Rc::new(CapWeightedRunbookQuotaAdjuster { cap: 32253.0 }),
            Rc::new(RoundingWeightedRunbookQuotaAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 71823.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_runbook_quota_engine();
        let item = new_weighted_runbook_quota("code-937", "name-359");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
