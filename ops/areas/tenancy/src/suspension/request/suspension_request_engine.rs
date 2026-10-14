use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the suspension request calculation.
pub trait SuspensionRequestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionRequestAdjuster {
    pub percent: f64,
}

impl SuspensionRequestAdjuster for PercentSuspensionRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionRequestAdjuster for FlaggedSuspensionRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRequestPortOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionRequestAdjuster for SuspensionRequestPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionRequest, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

/// Runs the suspension request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionRequestEngine {
    pub steps: Vec<Rc<dyn SuspensionRequestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionRequestEngine {
    pub fn evaluate(&mut self, item: &SuspensionRequest) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_suspension_request_engine(flags: &flags::Registry) -> SuspensionRequestEngine {
    SuspensionRequestEngine {
        steps: vec![
            Rc::new(PercentSuspensionRequestAdjuster { percent: -23.1 }) as Rc<dyn SuspensionRequestAdjuster>,
            Rc::new(FlaggedSuspensionRequestAdjuster {
                flags: flags.clone(),
                flag: "suspension-request-pending".to_string(),
                percent: -3.8,
            }),
            Rc::new(SuspensionRequestPortOffsetAdjuster { weight: 0.94 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 75011.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_request_engine(&flags::new_registry());
        let item = new_suspension_request("code-830", "label-980");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
