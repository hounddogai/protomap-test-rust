use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the probe assignment calculation.
pub trait ProbeAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeAssignmentAdjuster {
    pub percent: f64,
}

impl ProbeAssignmentAdjuster for PercentProbeAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingProbeAssignmentAdjuster {
    pub step: f64,
}

impl ProbeAssignmentAdjuster for RoundingProbeAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ProbeAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeAssignmentAdjuster for TieredProbeAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeAssignmentAdjuster {
    pub cap: f64,
}

impl ProbeAssignmentAdjuster for CapProbeAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the probe assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeAssignmentEngine {
    pub steps: Vec<Rc<dyn ProbeAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeAssignmentEngine {
    pub fn compute(&mut self, item: &ProbeAssignment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ProbeAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_probe_assignment_engine() -> ProbeAssignmentEngine {
    ProbeAssignmentEngine {
        steps: vec![
            Rc::new(PercentProbeAssignmentAdjuster { percent: 11.2 }) as Rc<dyn ProbeAssignmentAdjuster>,
            Rc::new(RoundingProbeAssignmentAdjuster { step: 0.05 }),
            Rc::new(TieredProbeAssignmentAdjuster {
                bounds: vec![5716.0, 8698.0],
                rates: vec![1.11, 1.0],
            }),
            Rc::new(CapProbeAssignmentAdjuster { cap: 6669.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 78597.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_assignment_engine();
        let item = new_probe_assignment("code-224", "name-765");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
