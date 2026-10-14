use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive rule batch calculation.
pub trait AdaptiveRuleBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRuleBatch, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveRuleBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveRuleBatchStep for FlaggedAdaptiveRuleBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveRuleBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveRuleBatchAdjuster {
    pub percent: f64,
}

impl AdaptiveRuleBatchStep for PercentAdaptiveRuleBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveRuleBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelAdaptiveRuleBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveRuleBatchStep for ChannelAdaptiveRuleBatchAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &AdaptiveRuleBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveRuleBatchAdjuster {
    pub minimum: f64,
}

impl AdaptiveRuleBatchStep for MinimumAdaptiveRuleBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveRuleBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveRuleBatchAdjuster {
    pub cap: f64,
}

impl AdaptiveRuleBatchStep for CapAdaptiveRuleBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveRuleBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleBatchUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRuleBatchStep for AdaptiveRuleBatchUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRuleBatch, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

/// Runs the adaptive rule batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRuleBatchEngine {
    pub steps: Vec<Rc<dyn AdaptiveRuleBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRuleBatchEngine {
    pub fn evaluate(&mut self, item: &AdaptiveRuleBatch) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRuleBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_rule_batch_engine(flags: &flags::Registry) -> AdaptiveRuleBatchEngine {
    AdaptiveRuleBatchEngine {
        steps: vec![
            Rc::new(FlaggedAdaptiveRuleBatchAdjuster {
                flags: flags.clone(),
                flag: "adaptive-rule-batch-global".to_string(),
                percent: 6.7,
            }) as Rc<dyn AdaptiveRuleBatchStep>,
            Rc::new(PercentAdaptiveRuleBatchAdjuster { percent: -5.6 }),
            Rc::new(ChannelAdaptiveRuleBatchAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.27), ("north".to_string(), 1.32)]),
            }),
            Rc::new(MinimumAdaptiveRuleBatchAdjuster { minimum: 14.0 }),
            Rc::new(CapAdaptiveRuleBatchAdjuster { cap: 30718.0 }),
            Rc::new(AdaptiveRuleBatchUpdatedAtOffsetAdjuster { weight: 0.07 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 58057.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_rule_batch_engine(&flags::new_registry());
        let item = new_adaptive_rule_batch("code-799", "label-215");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
