use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the canary target calculation.
pub trait CanaryTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryTargetModifier for TieredCanaryTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTargetExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl CanaryTargetModifier for CanaryTargetExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryTarget, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryTargetAdjuster {
    pub minimum: f64,
}

impl CanaryTargetModifier for MinimumCanaryTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCanaryTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanaryTargetModifier for FlaggedCanaryTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanaryTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the canary target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryTargetEngine {
    pub steps: Vec<Rc<dyn CanaryTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryTargetEngine {
    pub fn price(&mut self, item: &CanaryTarget) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CanaryTargetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CanaryTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_canary_target_engine(flags: &flags::Registry) -> CanaryTargetEngine {
    CanaryTargetEngine {
        steps: vec![
            Rc::new(TieredCanaryTargetAdjuster {
                bounds: vec![1836.0, 6354.0, 9450.0],
                rates: vec![0.89, 0.91, 0.81],
            }) as Rc<dyn CanaryTargetModifier>,
            Rc::new(CanaryTargetExpiresAtOffsetAdjuster { weight: 1.34 }),
            Rc::new(MinimumCanaryTargetAdjuster { minimum: 41.2 }),
            Rc::new(FlaggedCanaryTargetAdjuster {
                flags: flags.clone(),
                flag: "canary-target-tiered".to_string(),
                percent: -7.4,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 46501.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_target_engine(&flags::new_registry());
        let item = new_canary_target("code-329", "title-103");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
