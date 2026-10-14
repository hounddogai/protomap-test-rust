use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual probe request calculation.
pub trait ManualProbeRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualProbeRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualProbeRequestAdjuster {
    pub step: f64,
}

impl ManualProbeRequestStep for RoundingManualProbeRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualProbeRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceManualProbeRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualProbeRequestStep for SourceManualProbeRequestAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &ManualProbeRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualProbeRequestAdjuster {
    pub minimum: f64,
}

impl ManualProbeRequestStep for MinimumManualProbeRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualProbeRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeRequestCeilingOffsetAdjuster {
    pub weight: f64,
}

impl ManualProbeRequestStep for ManualProbeRequestCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &ManualProbeRequest, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

/// Runs the manual probe request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualProbeRequestEngine {
    pub steps: Vec<Rc<dyn ManualProbeRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualProbeRequestEngine {
    pub fn evaluate(&mut self, item: &ManualProbeRequest) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[ManualProbeRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_probe_request_engine() -> ManualProbeRequestEngine {
    ManualProbeRequestEngine {
        steps: vec![
            Rc::new(RoundingManualProbeRequestAdjuster { step: 0.1 }) as Rc<dyn ManualProbeRequestStep>,
            Rc::new(SourceManualProbeRequestAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.08),
                    ("west".to_string(), 1.27),
                    ("online".to_string(), 1.29),
                ]),
            }),
            Rc::new(MinimumManualProbeRequestAdjuster { minimum: 41.1 }),
            Rc::new(ManualProbeRequestCeilingOffsetAdjuster { weight: 1.84 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 49776.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_probe_request_engine();
        let item = new_manual_probe_request("code-551", "name-440");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
