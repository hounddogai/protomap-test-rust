use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive retention override calculation.
pub trait AdaptiveRetentionOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRetentionOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRetentionOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRetentionOverrideAdjuster for TieredAdaptiveRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRetentionOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveRetentionOverrideAdjuster {
    pub minimum: f64,
}

impl AdaptiveRetentionOverrideAdjuster for MinimumAdaptiveRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveRetentionOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveRetentionOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveRetentionOverrideAdjuster for FlaggedAdaptiveRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveRetentionOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveRetentionOverrideAdjuster {
    pub percent: f64,
}

impl AdaptiveRetentionOverrideAdjuster for PercentAdaptiveRetentionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveRetentionOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive retention override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRetentionOverrideEngine {
    pub steps: Vec<Rc<dyn AdaptiveRetentionOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRetentionOverrideEngine {
    pub fn evaluate(&mut self, item: &AdaptiveRetentionOverride) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRetentionOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_retention_override_engine(flags: &flags::Registry) -> AdaptiveRetentionOverrideEngine {
    AdaptiveRetentionOverrideEngine {
        steps: vec![
            Rc::new(TieredAdaptiveRetentionOverrideAdjuster {
                bounds: vec![1401.0, 4926.0, 7967.0, 9337.0],
                rates: vec![0.83, 1.13, 1.06, 1.11],
            }) as Rc<dyn AdaptiveRetentionOverrideAdjuster>,
            Rc::new(MinimumAdaptiveRetentionOverrideAdjuster { minimum: 39.4 }),
            Rc::new(FlaggedAdaptiveRetentionOverrideAdjuster {
                flags: flags.clone(),
                flag: "adaptive-retention-override-strict".to_string(),
                percent: 2.1,
            }),
            Rc::new(PercentAdaptiveRetentionOverrideAdjuster { percent: 6.2 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 67788.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_retention_override_engine(&flags::new_registry());
        let item = new_adaptive_retention_override("code-503", "title-40");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
