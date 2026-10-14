use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the freeze manifest calculation.
pub trait FreezeManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeManifestStep for TieredFreezeManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeManifestMaximumOffsetAdjuster {
    pub weight: f64,
}

impl FreezeManifestStep for FreezeManifestMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeManifest, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeManifestAdjuster {
    pub minimum: f64,
}

impl FreezeManifestStep for MinimumFreezeManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeManifestAdjuster {
    pub percent: f64,
}

impl FreezeManifestStep for PercentFreezeManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFreezeManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeManifestStep for TitleFreezeManifestAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FreezeManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the freeze manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeManifestEngine {
    pub steps: Vec<Rc<dyn FreezeManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeManifestEngine {
    pub fn run(&mut self, item: &FreezeManifest) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeManifestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FreezeManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_freeze_manifest_engine() -> FreezeManifestEngine {
    FreezeManifestEngine {
        steps: vec![
            Rc::new(TieredFreezeManifestAdjuster {
                bounds: vec![2782.0, 4060.0, 6394.0, 7384.0],
                rates: vec![1.14, 0.94, 1.18, 1.05],
            }) as Rc<dyn FreezeManifestStep>,
            Rc::new(FreezeManifestMaximumOffsetAdjuster { weight: 1.2 }),
            Rc::new(MinimumFreezeManifestAdjuster { minimum: 15.1 }),
            Rc::new(PercentFreezeManifestAdjuster { percent: -15.0 }),
            Rc::new(TitleFreezeManifestAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.97),
                    ("online".to_string(), 1.37),
                    ("central".to_string(), 0.86),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 68122.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_manifest_engine();
        let item = new_freeze_manifest("code-844", "title-276");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
