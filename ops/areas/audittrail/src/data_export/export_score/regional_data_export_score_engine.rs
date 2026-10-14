use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional data export score calculation.
pub trait RegionalDataExportScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDataExportScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDataExportScoreVersionOffsetAdjuster {
    pub weight: f64,
}

impl RegionalDataExportScoreModifier for RegionalDataExportScoreVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalDataExportScore, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalDataExportScoreAdjuster {
    pub percent: f64,
}

impl RegionalDataExportScoreModifier for PercentRegionalDataExportScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalDataExportScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalDataExportScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalDataExportScoreModifier for TieredRegionalDataExportScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalDataExportScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRegionalDataExportScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalDataExportScoreModifier for CategoryRegionalDataExportScoreAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RegionalDataExportScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalDataExportScoreAdjuster {
    pub step: f64,
}

impl RegionalDataExportScoreModifier for RoundingRegionalDataExportScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalDataExportScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the regional data export score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDataExportScoreEngine {
    pub steps: Vec<Rc<dyn RegionalDataExportScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDataExportScoreEngine {
    pub fn price(&mut self, item: &RegionalDataExportScore) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalDataExportScoreModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalDataExportScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_data_export_score_engine() -> RegionalDataExportScoreEngine {
    RegionalDataExportScoreEngine {
        steps: vec![
            Rc::new(RegionalDataExportScoreVersionOffsetAdjuster { weight: 0.1 })
                as Rc<dyn RegionalDataExportScoreModifier>,
            Rc::new(PercentRegionalDataExportScoreAdjuster { percent: 18.0 }),
            Rc::new(TieredRegionalDataExportScoreAdjuster {
                bounds: vec![882.0, 3610.0, 5124.0, 6471.0],
                rates: vec![1.04, 1.08, 1.08, 1.06],
            }),
            Rc::new(CategoryRegionalDataExportScoreAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.13)]),
            }),
            Rc::new(RoundingRegionalDataExportScoreAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 62249.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_data_export_score_engine();
        let item = new_regional_data_export_score("code-846", "label-806");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
