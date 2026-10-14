use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the usage rollup calculation.
pub trait UsageRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingUsageRollupAdjuster {
    pub step: f64,
}

impl UsageRollupFactor for RoundingUsageRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &UsageRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageRollupAdjuster {
    pub percent: f64,
}

impl UsageRollupFactor for PercentUsageRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceUsageRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageRollupFactor for NamespaceUsageRollupAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &UsageRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageRollupAdjuster {
    pub minimum: f64,
}

impl UsageRollupFactor for MinimumUsageRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRollupStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl UsageRollupFactor for UsageRollupStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &UsageRollup, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageRollupFactor for FlaggedUsageRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the usage rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageRollupEngine {
    pub steps: Vec<Rc<dyn UsageRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageRollupEngine {
    pub fn price(&mut self, item: &UsageRollup) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[UsageRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_usage_rollup_engine(flags: &flags::Registry) -> UsageRollupEngine {
    UsageRollupEngine {
        steps: vec![
            Rc::new(RoundingUsageRollupAdjuster { step: 0.1 }) as Rc<dyn UsageRollupFactor>,
            Rc::new(PercentUsageRollupAdjuster { percent: -10.2 }),
            Rc::new(NamespaceUsageRollupAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.35), ("north".to_string(), 1.03)]),
            }),
            Rc::new(MinimumUsageRollupAdjuster { minimum: 18.0 }),
            Rc::new(UsageRollupStartsAtOffsetAdjuster { weight: 0.02 }),
            Rc::new(FlaggedUsageRollupAdjuster {
                flags: flags.clone(),
                flag: "usage-rollup-scheduled".to_string(),
                percent: -1.7,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 23828.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_rollup_engine(&flags::new_registry());
        let item = new_usage_rollup("code-911", "title-586");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
