use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the usage override calculation.
pub trait UsageOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsageOverrideStep for TieredUsageOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsageOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageOverrideAdjuster {
    pub minimum: f64,
}

impl UsageOverrideStep for MinimumUsageOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryUsageOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageOverrideStep for CategoryUsageOverrideAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &UsageOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the usage override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageOverrideEngine {
    pub steps: Vec<Rc<dyn UsageOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageOverrideEngine {
    pub fn evaluate(&mut self, item: &UsageOverride) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[UsageOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_usage_override_engine() -> UsageOverrideEngine {
    UsageOverrideEngine {
        steps: vec![
            Rc::new(TieredUsageOverrideAdjuster {
                bounds: vec![115.0, 3402.0, 4780.0, 7314.0],
                rates: vec![0.93, 1.12, 1.18, 0.91],
            }) as Rc<dyn UsageOverrideStep>,
            Rc::new(MinimumUsageOverrideAdjuster { minimum: 6.5 }),
            Rc::new(CategoryUsageOverrideAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.93), ("west".to_string(), 0.96)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 49771.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_override_engine();
        let item = new_usage_override("code-358", "label-727");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
