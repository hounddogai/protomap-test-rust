use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the seasonal postmortem manifest calculation.
pub trait SeasonalPostmortemManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalPostmortemManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSeasonalPostmortemManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalPostmortemManifestFactor for NameSeasonalPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SeasonalPostmortemManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalPostmortemManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalPostmortemManifestFactor for TieredSeasonalPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalPostmortemManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalPostmortemManifestAdjuster {
    pub percent: f64,
}

impl SeasonalPostmortemManifestFactor for PercentSeasonalPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalPostmortemManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal postmortem manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalPostmortemManifestEngine {
    pub steps: Vec<Rc<dyn SeasonalPostmortemManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalPostmortemManifestEngine {
    pub fn compute(&mut self, item: &SeasonalPostmortemManifest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalPostmortemManifestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalPostmortemManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_postmortem_manifest_engine() -> SeasonalPostmortemManifestEngine {
    SeasonalPostmortemManifestEngine {
        steps: vec![
            Rc::new(NameSeasonalPostmortemManifestAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.11)]),
            }) as Rc<dyn SeasonalPostmortemManifestFactor>,
            Rc::new(TieredSeasonalPostmortemManifestAdjuster {
                bounds: vec![255.0, 638.0, 2524.0, 5360.0],
                rates: vec![1.18, 0.86, 1.04, 1.11],
            }),
            Rc::new(PercentSeasonalPostmortemManifestAdjuster { percent: 10.3 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 44852.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_postmortem_manifest_engine();
        let item = new_seasonal_postmortem_manifest("code-284", "name-637");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
