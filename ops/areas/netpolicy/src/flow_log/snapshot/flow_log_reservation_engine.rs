use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the flow log reservation calculation.
pub trait FlowLogReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogReservationAdjuster {
    pub percent: f64,
}

impl FlowLogReservationModifier for PercentFlowLogReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FlowLogReservationModifier for TieredFlowLogReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FlowLogReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogReservationAdjuster {
    pub step: f64,
}

impl FlowLogReservationModifier for RoundingFlowLogReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogReservationAdjuster {
    pub cap: f64,
}

impl FlowLogReservationModifier for CapFlowLogReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the flow log reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogReservationEngine {
    pub steps: Vec<Rc<dyn FlowLogReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogReservationEngine {
    pub fn price(&mut self, item: &FlowLogReservation) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_flow_log_reservation_engine() -> FlowLogReservationEngine {
    FlowLogReservationEngine {
        steps: vec![
            Rc::new(PercentFlowLogReservationAdjuster { percent: 19.2 }) as Rc<dyn FlowLogReservationModifier>,
            Rc::new(TieredFlowLogReservationAdjuster {
                bounds: vec![2078.0, 4892.0, 5265.0, 8888.0],
                rates: vec![1.03, 0.9, 1.15, 0.92],
            }),
            Rc::new(RoundingFlowLogReservationAdjuster { step: 0.05 }),
            Rc::new(CapFlowLogReservationAdjuster { cap: 47927.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 93259.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_reservation_engine();
        let item = new_flow_log_reservation("code-862", "title-901");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
