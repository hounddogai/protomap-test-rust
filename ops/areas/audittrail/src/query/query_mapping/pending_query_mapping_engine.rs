use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending query mapping calculation.
pub trait PendingQueryMappingAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingQueryMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingQueryMappingAdjuster {
    pub percent: f64,
}

impl PendingQueryMappingAdjuster for PercentPendingQueryMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingQueryMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingQueryMappingAdjuster {
    pub step: f64,
}

impl PendingQueryMappingAdjuster for RoundingPendingQueryMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingQueryMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingQueryMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingQueryMappingAdjuster for FlaggedPendingQueryMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingQueryMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending query mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingQueryMappingEngine {
    pub steps: Vec<Rc<dyn PendingQueryMappingAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingQueryMappingEngine {
    pub fn run(&mut self, item: &PendingQueryMapping) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[PendingQueryMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_query_mapping_engine(flags: &flags::Registry) -> PendingQueryMappingEngine {
    PendingQueryMappingEngine {
        steps: vec![
            Rc::new(PercentPendingQueryMappingAdjuster { percent: -22.4 }) as Rc<dyn PendingQueryMappingAdjuster>,
            Rc::new(RoundingPendingQueryMappingAdjuster { step: 0.1 }),
            Rc::new(FlaggedPendingQueryMappingAdjuster {
                flags: flags.clone(),
                flag: "pending-query-mapping-adaptive".to_string(),
                percent: -7.4,
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 30445.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_query_mapping_engine(&flags::new_registry());
        let item = new_pending_query_mapping("code-233", "label-796");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
