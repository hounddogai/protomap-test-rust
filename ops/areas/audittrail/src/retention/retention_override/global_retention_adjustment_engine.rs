use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global retention adjustment calculation.
pub trait GlobalRetentionAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalRetentionAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalRetentionAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalRetentionAdjustmentFactor for FlaggedGlobalRetentionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalRetentionAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionAdjustmentWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl GlobalRetentionAdjustmentFactor for GlobalRetentionAdjustmentWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalRetentionAdjustment, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalRetentionAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalRetentionAdjustmentFactor for TieredGlobalRetentionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalRetentionAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalRetentionAdjustmentAdjuster {
    pub cap: f64,
}

impl GlobalRetentionAdjustmentFactor for CapGlobalRetentionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalRetentionAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelGlobalRetentionAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalRetentionAdjustmentFactor for LabelGlobalRetentionAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &GlobalRetentionAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalRetentionAdjustmentAdjuster {
    pub step: f64,
}

impl GlobalRetentionAdjustmentFactor for RoundingGlobalRetentionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalRetentionAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global retention adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalRetentionAdjustmentEngine {
    pub steps: Vec<Rc<dyn GlobalRetentionAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalRetentionAdjustmentEngine {
    pub fn compute(&mut self, item: &GlobalRetentionAdjustment) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalRetentionAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_retention_adjustment_engine(flags: &flags::Registry) -> GlobalRetentionAdjustmentEngine {
    GlobalRetentionAdjustmentEngine {
        steps: vec![
            Rc::new(FlaggedGlobalRetentionAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "global-retention-adjustment-batched".to_string(),
                percent: -5.5,
            }) as Rc<dyn GlobalRetentionAdjustmentFactor>,
            Rc::new(GlobalRetentionAdjustmentWindowDaysOffsetAdjuster { weight: 0.36 }),
            Rc::new(TieredGlobalRetentionAdjustmentAdjuster {
                bounds: vec![5846.0, 6233.0, 6820.0, 7416.0],
                rates: vec![1.16, 1.16, 0.94, 0.88],
            }),
            Rc::new(CapGlobalRetentionAdjustmentAdjuster { cap: 42757.0 }),
            Rc::new(LabelGlobalRetentionAdjustmentAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.91),
                    ("online".to_string(), 0.91),
                    ("retail".to_string(), 1.02),
                ]),
            }),
            Rc::new(RoundingGlobalRetentionAdjustmentAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 63346.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_retention_adjustment_engine(&flags::new_registry());
        let item = new_global_retention_adjustment("code-938", "label-504");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
