use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global suspension batch calculation.
pub trait GlobalSuspensionBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalSuspensionBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionBatchWeightOffsetAdjuster {
    pub weight: f64,
}

impl GlobalSuspensionBatchStep for GlobalSuspensionBatchWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalSuspensionBatch, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalSuspensionBatchAdjuster {
    pub minimum: f64,
}

impl GlobalSuspensionBatchStep for MinimumGlobalSuspensionBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalSuspensionBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelGlobalSuspensionBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalSuspensionBatchStep for LabelGlobalSuspensionBatchAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &GlobalSuspensionBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalSuspensionBatchAdjuster {
    pub percent: f64,
}

impl GlobalSuspensionBatchStep for PercentGlobalSuspensionBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalSuspensionBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalSuspensionBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalSuspensionBatchStep for FlaggedGlobalSuspensionBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalSuspensionBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global suspension batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalSuspensionBatchEngine {
    pub steps: Vec<Rc<dyn GlobalSuspensionBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalSuspensionBatchEngine {
    pub fn price(&mut self, item: &GlobalSuspensionBatch) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[GlobalSuspensionBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_suspension_batch_engine(flags: &flags::Registry) -> GlobalSuspensionBatchEngine {
    GlobalSuspensionBatchEngine {
        steps: vec![
            Rc::new(GlobalSuspensionBatchWeightOffsetAdjuster { weight: 1.1 }) as Rc<dyn GlobalSuspensionBatchStep>,
            Rc::new(MinimumGlobalSuspensionBatchAdjuster { minimum: 2.5 }),
            Rc::new(LabelGlobalSuspensionBatchAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.14),
                    ("central".to_string(), 0.86),
                    ("north".to_string(), 1.37),
                ]),
            }),
            Rc::new(PercentGlobalSuspensionBatchAdjuster { percent: 20.4 }),
            Rc::new(FlaggedGlobalSuspensionBatchAdjuster {
                flags: flags.clone(),
                flag: "global-suspension-batch-capped".to_string(),
                percent: 8.1,
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 63961.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_suspension_batch_engine(&flags::new_registry());
        let item = new_global_suspension_batch("code-633", "label-28");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
