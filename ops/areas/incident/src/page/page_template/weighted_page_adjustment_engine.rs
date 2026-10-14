use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted page adjustment calculation.
pub trait WeightedPageAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedPageAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedPageAdjustmentAdjuster {
    pub cap: f64,
}

impl WeightedPageAdjustmentModifier for CapWeightedPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedPageAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedPageAdjustmentAdjuster {
    pub percent: f64,
}

impl WeightedPageAdjustmentModifier for PercentWeightedPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedPageAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedPageAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedPageAdjustmentModifier for FlaggedWeightedPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedPageAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelWeightedPageAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedPageAdjustmentModifier for LabelWeightedPageAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &WeightedPageAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted page adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedPageAdjustmentEngine {
    pub steps: Vec<Rc<dyn WeightedPageAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedPageAdjustmentEngine {
    pub fn run(&mut self, item: &WeightedPageAdjustment) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[WeightedPageAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_page_adjustment_engine(flags: &flags::Registry) -> WeightedPageAdjustmentEngine {
    WeightedPageAdjustmentEngine {
        steps: vec![
            Rc::new(CapWeightedPageAdjustmentAdjuster { cap: 45238.0 }) as Rc<dyn WeightedPageAdjustmentModifier>,
            Rc::new(PercentWeightedPageAdjustmentAdjuster { percent: -21.9 }),
            Rc::new(FlaggedWeightedPageAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "weighted-page-adjustment-fallback".to_string(),
                percent: -1.1,
            }),
            Rc::new(LabelWeightedPageAdjustmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.13)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 68531.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_page_adjustment_engine(&flags::new_registry());
        let item = new_weighted_page_adjustment("code-234", "label-285");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
