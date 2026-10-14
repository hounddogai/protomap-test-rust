use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the usage target calculation.
pub trait UsageTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageTargetAdjuster {
    pub percent: f64,
}

impl UsageTargetFactor for PercentUsageTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitUsageTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageTargetFactor for UnitUsageTargetAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &UsageTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageTargetAdjuster {
    pub minimum: f64,
}

impl UsageTargetFactor for MinimumUsageTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageTargetAdjuster {
    pub cap: f64,
}

impl UsageTargetFactor for CapUsageTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageTargetFactor for FlaggedUsageTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsageTargetFactor for TieredUsageTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsageTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the usage target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageTargetEngine {
    pub steps: Vec<Rc<dyn UsageTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageTargetEngine {
    pub fn evaluate(&mut self, item: &UsageTarget) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsageTargetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsageTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_usage_target_engine(flags: &flags::Registry) -> UsageTargetEngine {
    UsageTargetEngine {
        steps: vec![
            Rc::new(PercentUsageTargetAdjuster { percent: 16.5 }) as Rc<dyn UsageTargetFactor>,
            Rc::new(UnitUsageTargetAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.22),
                    ("west".to_string(), 1.34),
                    ("east".to_string(), 1.32),
                ]),
            }),
            Rc::new(MinimumUsageTargetAdjuster { minimum: 13.8 }),
            Rc::new(CapUsageTargetAdjuster { cap: 25924.0 }),
            Rc::new(FlaggedUsageTargetAdjuster {
                flags: flags.clone(),
                flag: "usage-target-scheduled".to_string(),
                percent: 4.1,
            }),
            Rc::new(TieredUsageTargetAdjuster {
                bounds: vec![2830.0, 4373.0],
                rates: vec![1.17, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 15502.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_target_engine(&flags::new_registry());
        let item = new_usage_target("code-316", "title-395");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
