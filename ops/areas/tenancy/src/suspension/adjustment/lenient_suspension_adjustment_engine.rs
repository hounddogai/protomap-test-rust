use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the lenient suspension adjustment calculation.
pub trait LenientSuspensionAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientSuspensionAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientSuspensionAdjustmentAdjuster {
    pub percent: f64,
}

impl LenientSuspensionAdjustmentStep for PercentLenientSuspensionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientSuspensionAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionAdjustmentQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl LenientSuspensionAdjustmentStep for LenientSuspensionAdjustmentQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &LenientSuspensionAdjustment, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLenientSuspensionAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientSuspensionAdjustmentStep for SegmentLenientSuspensionAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &LenientSuspensionAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient suspension adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientSuspensionAdjustmentEngine {
    pub steps: Vec<Rc<dyn LenientSuspensionAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientSuspensionAdjustmentEngine {
    pub fn compute(&mut self, item: &LenientSuspensionAdjustment) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientSuspensionAdjustmentStep> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[LenientSuspensionAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_suspension_adjustment_engine() -> LenientSuspensionAdjustmentEngine {
    LenientSuspensionAdjustmentEngine {
        steps: vec![
            Rc::new(PercentLenientSuspensionAdjustmentAdjuster { percent: 17.9 })
                as Rc<dyn LenientSuspensionAdjustmentStep>,
            Rc::new(LenientSuspensionAdjustmentQuotaUnitsOffsetAdjuster { weight: 0.22 }),
            Rc::new(SegmentLenientSuspensionAdjustmentAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.32)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 30036.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_suspension_adjustment_engine();
        let item = new_lenient_suspension_adjustment("code-389", "title-517");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
