use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy retention estimate calculation.
pub trait LegacyRetentionEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRetentionEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLegacyRetentionEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyRetentionEstimateFactor for NameLegacyRetentionEstimateAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LegacyRetentionEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyRetentionEstimateAdjuster {
    pub cap: f64,
}

impl LegacyRetentionEstimateFactor for CapLegacyRetentionEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyRetentionEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyRetentionEstimateAdjuster {
    pub step: f64,
}

impl LegacyRetentionEstimateFactor for RoundingLegacyRetentionEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyRetentionEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionEstimateCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl LegacyRetentionEstimateFactor for LegacyRetentionEstimateCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyRetentionEstimate, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyRetentionEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyRetentionEstimateFactor for TieredLegacyRetentionEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyRetentionEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyRetentionEstimateAdjuster {
    pub minimum: f64,
}

impl LegacyRetentionEstimateFactor for MinimumLegacyRetentionEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyRetentionEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy retention estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRetentionEstimateEngine {
    pub steps: Vec<Rc<dyn LegacyRetentionEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRetentionEstimateEngine {
    pub fn price(&mut self, item: &LegacyRetentionEstimate) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[LegacyRetentionEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_retention_estimate_engine() -> LegacyRetentionEstimateEngine {
    LegacyRetentionEstimateEngine {
        steps: vec![
            Rc::new(NameLegacyRetentionEstimateAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.93),
                    ("south".to_string(), 1.24),
                    ("west".to_string(), 0.83),
                ]),
            }) as Rc<dyn LegacyRetentionEstimateFactor>,
            Rc::new(CapLegacyRetentionEstimateAdjuster { cap: 10169.0 }),
            Rc::new(RoundingLegacyRetentionEstimateAdjuster { step: 0.5 }),
            Rc::new(LegacyRetentionEstimateCreatedAtOffsetAdjuster { weight: 0.4 }),
            Rc::new(TieredLegacyRetentionEstimateAdjuster {
                bounds: vec![178.0, 2108.0, 9303.0],
                rates: vec![1.09, 1.05, 1.16],
            }),
            Rc::new(MinimumLegacyRetentionEstimateAdjuster { minimum: 0.9 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 36804.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_retention_estimate_engine();
        let item = new_legacy_retention_estimate("code-838", "name-885");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
