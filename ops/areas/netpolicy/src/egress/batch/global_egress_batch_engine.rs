use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global egress batch calculation.
pub trait GlobalEgressBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalEgressBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalEgressBatchAdjuster {
    pub minimum: f64,
}

impl GlobalEgressBatchModifier for MinimumGlobalEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalEgressBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalEgressBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalEgressBatchModifier for FlaggedGlobalEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalEgressBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalEgressBatchAdjuster {
    pub percent: f64,
}

impl GlobalEgressBatchModifier for PercentGlobalEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalEgressBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalEgressBatchAdjuster {
    pub cap: f64,
}

impl GlobalEgressBatchModifier for CapGlobalEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalEgressBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalEgressBatchAdjuster {
    pub step: f64,
}

impl GlobalEgressBatchModifier for RoundingGlobalEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalEgressBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelGlobalEgressBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalEgressBatchModifier for LabelGlobalEgressBatchAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &GlobalEgressBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the global egress batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalEgressBatchEngine {
    pub steps: Vec<Rc<dyn GlobalEgressBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalEgressBatchEngine {
    pub fn price(&mut self, item: &GlobalEgressBatch) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[GlobalEgressBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_egress_batch_engine(flags: &flags::Registry) -> GlobalEgressBatchEngine {
    GlobalEgressBatchEngine {
        steps: vec![
            Rc::new(MinimumGlobalEgressBatchAdjuster { minimum: 40.5 }) as Rc<dyn GlobalEgressBatchModifier>,
            Rc::new(FlaggedGlobalEgressBatchAdjuster {
                flags: flags.clone(),
                flag: "global-egress-batch-lenient".to_string(),
                percent: 8.2,
            }),
            Rc::new(PercentGlobalEgressBatchAdjuster { percent: 22.8 }),
            Rc::new(CapGlobalEgressBatchAdjuster { cap: 9631.0 }),
            Rc::new(RoundingGlobalEgressBatchAdjuster { step: 1.0 }),
            Rc::new(LabelGlobalEgressBatchAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.96)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 27690.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_egress_batch_engine(&flags::new_registry());
        let item = new_global_egress_batch("code-153", "label-177");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
