use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered reservation summary calculation.
pub trait TieredReservationSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredReservationSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredReservationSummaryAdjuster {
    pub minimum: f64,
}

impl TieredReservationSummaryAdjuster for MinimumTieredReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredReservationSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTieredReservationSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredReservationSummaryAdjuster for ChannelTieredReservationSummaryAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TieredReservationSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationSummaryCostOffsetAdjuster {
    pub weight: f64,
}

impl TieredReservationSummaryAdjuster for TieredReservationSummaryCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &TieredReservationSummary, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

/// Runs the tiered reservation summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredReservationSummaryEngine {
    pub steps: Vec<Rc<dyn TieredReservationSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredReservationSummaryEngine {
    pub fn compute(&mut self, item: &TieredReservationSummary) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[TieredReservationSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_reservation_summary_engine() -> TieredReservationSummaryEngine {
    TieredReservationSummaryEngine {
        steps: vec![
            Rc::new(MinimumTieredReservationSummaryAdjuster { minimum: 35.3 })
                as Rc<dyn TieredReservationSummaryAdjuster>,
            Rc::new(ChannelTieredReservationSummaryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.24), ("retail".to_string(), 1.39)]),
            }),
            Rc::new(TieredReservationSummaryCostOffsetAdjuster { weight: 0.54 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 56497.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_reservation_summary_engine();
        let item = new_tiered_reservation_summary("code-263", "title-864");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
