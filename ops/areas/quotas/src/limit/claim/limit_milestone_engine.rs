use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the limit milestone calculation.
pub trait LimitMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryLimitMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LimitMilestoneModifier for CategoryLimitMilestoneAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &LimitMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitMilestoneAdjuster {
    pub minimum: f64,
}

impl LimitMilestoneModifier for MinimumLimitMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitMilestoneModifier for TieredLimitMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLimitMilestoneAdjuster {
    pub percent: f64,
}

impl LimitMilestoneModifier for PercentLimitMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LimitMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLimitMilestoneAdjuster {
    pub cap: f64,
}

impl LimitMilestoneModifier for CapLimitMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LimitMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLimitMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LimitMilestoneModifier for FlaggedLimitMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LimitMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the limit milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitMilestoneEngine {
    pub steps: Vec<Rc<dyn LimitMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitMilestoneEngine {
    pub fn compute(&mut self, item: &LimitMilestone) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[LimitMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_limit_milestone_engine(flags: &flags::Registry) -> LimitMilestoneEngine {
    LimitMilestoneEngine {
        steps: vec![
            Rc::new(CategoryLimitMilestoneAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.79), ("north".to_string(), 1.34)]),
            }) as Rc<dyn LimitMilestoneModifier>,
            Rc::new(MinimumLimitMilestoneAdjuster { minimum: 34.7 }),
            Rc::new(TieredLimitMilestoneAdjuster {
                bounds: vec![1502.0, 6316.0, 7500.0, 9672.0],
                rates: vec![1.05, 0.81, 0.9, 0.97],
            }),
            Rc::new(PercentLimitMilestoneAdjuster { percent: -18.2 }),
            Rc::new(CapLimitMilestoneAdjuster { cap: 45392.0 }),
            Rc::new(FlaggedLimitMilestoneAdjuster {
                flags: flags.clone(),
                flag: "limit-milestone-legacy".to_string(),
                percent: -7.3,
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 46966.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_milestone_engine(&flags::new_registry());
        let item = new_limit_milestone("code-840", "label-561");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
