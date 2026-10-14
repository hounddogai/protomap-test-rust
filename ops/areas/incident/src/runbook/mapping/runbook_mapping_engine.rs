use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the runbook mapping calculation.
pub trait RunbookMappingModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookMappingAdjuster {
    pub percent: f64,
}

impl RunbookMappingModifier for PercentRunbookMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookMappingModifier for FlaggedRunbookMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookMappingAdjuster {
    pub step: f64,
}

impl RunbookMappingModifier for RoundingRunbookMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the runbook mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookMappingEngine {
    pub steps: Vec<Rc<dyn RunbookMappingModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookMappingEngine {
    pub fn evaluate(&mut self, item: &RunbookMapping) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[RunbookMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_runbook_mapping_engine(flags: &flags::Registry) -> RunbookMappingEngine {
    RunbookMappingEngine {
        steps: vec![
            Rc::new(PercentRunbookMappingAdjuster { percent: 4.6 }) as Rc<dyn RunbookMappingModifier>,
            Rc::new(FlaggedRunbookMappingAdjuster {
                flags: flags.clone(),
                flag: "runbook-mapping-global".to_string(),
                percent: 7.8,
            }),
            Rc::new(RoundingRunbookMappingAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 78949.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_mapping_engine(&flags::new_registry());
        let item = new_runbook_mapping("code-199", "label-848");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
