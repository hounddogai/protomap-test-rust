use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the failover request calculation.
pub trait FailoverRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRequestBurstOffsetAdjuster {
    pub weight: f64,
}

impl FailoverRequestFactor for FailoverRequestBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverRequest, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverRequestFactor for FlaggedFailoverRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverRequestFactor for TieredFailoverRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverRequestAdjuster {
    pub step: f64,
}

impl FailoverRequestFactor for RoundingFailoverRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFailoverRequestAdjuster {
    pub percent: f64,
}

impl FailoverRequestFactor for PercentFailoverRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FailoverRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the failover request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverRequestEngine {
    pub steps: Vec<Rc<dyn FailoverRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverRequestEngine {
    pub fn compute(&mut self, item: &FailoverRequest) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[FailoverRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_failover_request_engine(flags: &flags::Registry) -> FailoverRequestEngine {
    FailoverRequestEngine {
        steps: vec![
            Rc::new(FailoverRequestBurstOffsetAdjuster { weight: 1.21 }) as Rc<dyn FailoverRequestFactor>,
            Rc::new(FlaggedFailoverRequestAdjuster {
                flags: flags.clone(),
                flag: "failover-request-rolling".to_string(),
                percent: -2.7,
            }),
            Rc::new(TieredFailoverRequestAdjuster {
                bounds: vec![1241.0, 5945.0],
                rates: vec![0.81, 1.11],
            }),
            Rc::new(RoundingFailoverRequestAdjuster { step: 0.05 }),
            Rc::new(PercentFailoverRequestAdjuster { percent: -9.6 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 61964.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_request_engine(&flags::new_registry());
        let item = new_failover_request("code-66", "label-559");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
