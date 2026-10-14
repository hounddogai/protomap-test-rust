use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient migration threshold calculation.
pub trait LenientMigrationThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientMigrationThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeLenientMigrationThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientMigrationThresholdAdjuster for ScopeLenientMigrationThresholdAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &LenientMigrationThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationThresholdVersionOffsetAdjuster {
    pub weight: f64,
}

impl LenientMigrationThresholdAdjuster for LenientMigrationThresholdVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &LenientMigrationThreshold, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientMigrationThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientMigrationThresholdAdjuster for TieredLenientMigrationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientMigrationThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientMigrationThresholdAdjuster {
    pub cap: f64,
}

impl LenientMigrationThresholdAdjuster for CapLenientMigrationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientMigrationThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientMigrationThresholdAdjuster {
    pub step: f64,
}

impl LenientMigrationThresholdAdjuster for RoundingLenientMigrationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientMigrationThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientMigrationThresholdAdjuster {
    pub minimum: f64,
}

impl LenientMigrationThresholdAdjuster for MinimumLenientMigrationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientMigrationThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient migration threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientMigrationThresholdEngine {
    pub steps: Vec<Rc<dyn LenientMigrationThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientMigrationThresholdEngine {
    pub fn run(&mut self, item: &LenientMigrationThreshold) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[LenientMigrationThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_migration_threshold_engine() -> LenientMigrationThresholdEngine {
    LenientMigrationThresholdEngine {
        steps: vec![
            Rc::new(ScopeLenientMigrationThresholdAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.76),
                    ("east".to_string(), 0.89),
                    ("north".to_string(), 0.78),
                ]),
            }) as Rc<dyn LenientMigrationThresholdAdjuster>,
            Rc::new(LenientMigrationThresholdVersionOffsetAdjuster { weight: 1.77 }),
            Rc::new(TieredLenientMigrationThresholdAdjuster {
                bounds: vec![351.0, 7798.0, 9616.0],
                rates: vec![0.91, 1.11, 1.14],
            }),
            Rc::new(CapLenientMigrationThresholdAdjuster { cap: 7404.0 }),
            Rc::new(RoundingLenientMigrationThresholdAdjuster { step: 0.05 }),
            Rc::new(MinimumLenientMigrationThresholdAdjuster { minimum: 7.2 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 3074.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_migration_threshold_engine();
        let item = new_lenient_migration_threshold("code-533", "label-349");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
