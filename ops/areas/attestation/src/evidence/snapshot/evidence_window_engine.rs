use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence window calculation.
pub trait EvidenceWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceWindowAdjuster {
    pub minimum: f64,
}

impl EvidenceWindowAdjuster for MinimumEvidenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceWindowAdjuster {
    pub percent: f64,
}

impl EvidenceWindowAdjuster for PercentEvidenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceWindowAdjuster {
    pub cap: f64,
}

impl EvidenceWindowAdjuster for CapEvidenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceWindowAdjuster {
    pub step: f64,
}

impl EvidenceWindowAdjuster for RoundingEvidenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceWindowAdjuster for FlaggedEvidenceWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeEvidenceWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceWindowAdjuster for ScopeEvidenceWindowAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &EvidenceWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the evidence window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceWindowEngine {
    pub steps: Vec<Rc<dyn EvidenceWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceWindowEngine {
    pub fn run(&mut self, item: &EvidenceWindow) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[EvidenceWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_evidence_window_engine(flags: &flags::Registry) -> EvidenceWindowEngine {
    EvidenceWindowEngine {
        steps: vec![
            Rc::new(MinimumEvidenceWindowAdjuster { minimum: 37.1 }) as Rc<dyn EvidenceWindowAdjuster>,
            Rc::new(PercentEvidenceWindowAdjuster { percent: 0.0 }),
            Rc::new(CapEvidenceWindowAdjuster { cap: 11925.0 }),
            Rc::new(RoundingEvidenceWindowAdjuster { step: 0.05 }),
            Rc::new(FlaggedEvidenceWindowAdjuster {
                flags: flags.clone(),
                flag: "evidence-window-manual".to_string(),
                percent: -2.0,
            }),
            Rc::new(ScopeEvidenceWindowAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.76), ("online".to_string(), 1.37)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 12455.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_window_engine(&flags::new_registry());
        let item = new_evidence_window("code-753", "label-408");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
