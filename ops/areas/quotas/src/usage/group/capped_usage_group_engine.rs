use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped usage group calculation.
pub trait CappedUsageGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedUsageGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedUsageGroupAdjuster {
    pub step: f64,
}

impl CappedUsageGroupFactor for RoundingCappedUsageGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedUsageGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedUsageGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedUsageGroupFactor for FlaggedCappedUsageGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedUsageGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedUsageGroupSequenceOffsetAdjuster {
    pub weight: f64,
}

impl CappedUsageGroupFactor for CappedUsageGroupSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &CappedUsageGroup, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedUsageGroupAdjuster {
    pub percent: f64,
}

impl CappedUsageGroupFactor for PercentCappedUsageGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedUsageGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped usage group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedUsageGroupEngine {
    pub steps: Vec<Rc<dyn CappedUsageGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedUsageGroupEngine {
    pub fn evaluate(&mut self, item: &CappedUsageGroup) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedUsageGroupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedUsageGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_usage_group_engine(flags: &flags::Registry) -> CappedUsageGroupEngine {
    CappedUsageGroupEngine {
        steps: vec![
            Rc::new(RoundingCappedUsageGroupAdjuster { step: 1.0 }) as Rc<dyn CappedUsageGroupFactor>,
            Rc::new(FlaggedCappedUsageGroupAdjuster {
                flags: flags.clone(),
                flag: "capped-usage-group-primary".to_string(),
                percent: 2.9,
            }),
            Rc::new(CappedUsageGroupSequenceOffsetAdjuster { weight: 1.86 }),
            Rc::new(PercentCappedUsageGroupAdjuster { percent: -1.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 37751.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_usage_group_engine(&flags::new_registry());
        let item = new_capped_usage_group("code-690", "label-47");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
