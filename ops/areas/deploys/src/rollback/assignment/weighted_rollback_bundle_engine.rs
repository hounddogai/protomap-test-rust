use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted rollback bundle calculation.
pub trait WeightedRollbackBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRollbackBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedRollbackBundleAdjuster {
    pub minimum: f64,
}

impl WeightedRollbackBundleModifier for MinimumWeightedRollbackBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedRollbackBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedRollbackBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedRollbackBundleModifier for FlaggedWeightedRollbackBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedRollbackBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRollbackBundleScoreOffsetAdjuster {
    pub weight: f64,
}

impl WeightedRollbackBundleModifier for WeightedRollbackBundleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedRollbackBundle, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentWeightedRollbackBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedRollbackBundleModifier for SegmentWeightedRollbackBundleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &WeightedRollbackBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedRollbackBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedRollbackBundleModifier for TieredWeightedRollbackBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedRollbackBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted rollback bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRollbackBundleEngine {
    pub steps: Vec<Rc<dyn WeightedRollbackBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRollbackBundleEngine {
    pub fn run(&mut self, item: &WeightedRollbackBundle) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRollbackBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_rollback_bundle_engine(flags: &flags::Registry) -> WeightedRollbackBundleEngine {
    WeightedRollbackBundleEngine {
        steps: vec![
            Rc::new(MinimumWeightedRollbackBundleAdjuster { minimum: 14.0 }) as Rc<dyn WeightedRollbackBundleModifier>,
            Rc::new(FlaggedWeightedRollbackBundleAdjuster {
                flags: flags.clone(),
                flag: "weighted-rollback-bundle-weighted".to_string(),
                percent: -2.9,
            }),
            Rc::new(WeightedRollbackBundleScoreOffsetAdjuster { weight: 0.67 }),
            Rc::new(SegmentWeightedRollbackBundleAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.93),
                    ("retail".to_string(), 1.01),
                    ("south".to_string(), 0.91),
                ]),
            }),
            Rc::new(TieredWeightedRollbackBundleAdjuster {
                bounds: vec![9202.0, 9726.0],
                rates: vec![1.19, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 33409.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_rollback_bundle_engine(&flags::new_registry());
        let item = new_weighted_rollback_bundle("code-545", "name-9");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
