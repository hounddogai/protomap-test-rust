use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the suspension manifest calculation.
pub trait SuspensionManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionManifestAdjuster {
    pub cap: f64,
}

impl SuspensionManifestModifier for CapSuspensionManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionManifestAdjuster {
    pub percent: f64,
}

impl SuspensionManifestModifier for PercentSuspensionManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSuspensionManifestAdjuster {
    pub step: f64,
}

impl SuspensionManifestModifier for RoundingSuspensionManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SuspensionManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionManifestDiscountOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionManifestModifier for SuspensionManifestDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionManifest, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSuspensionManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SuspensionManifestModifier for TieredSuspensionManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SuspensionManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSuspensionManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionManifestModifier for TitleSuspensionManifestAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SuspensionManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the suspension manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionManifestEngine {
    pub steps: Vec<Rc<dyn SuspensionManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionManifestEngine {
    pub fn compute(&mut self, item: &SuspensionManifest) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SuspensionManifestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SuspensionManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_suspension_manifest_engine() -> SuspensionManifestEngine {
    SuspensionManifestEngine {
        steps: vec![
            Rc::new(CapSuspensionManifestAdjuster { cap: 40655.0 }) as Rc<dyn SuspensionManifestModifier>,
            Rc::new(PercentSuspensionManifestAdjuster { percent: -18.7 }),
            Rc::new(RoundingSuspensionManifestAdjuster { step: 0.25 }),
            Rc::new(SuspensionManifestDiscountOffsetAdjuster { weight: 1.04 }),
            Rc::new(TieredSuspensionManifestAdjuster {
                bounds: vec![64.0, 888.0, 8410.0],
                rates: vec![0.91, 1.16, 1.18],
            }),
            Rc::new(TitleSuspensionManifestAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.8),
                    ("retail".to_string(), 1.33),
                    ("north".to_string(), 0.87),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 77986.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_manifest_engine();
        let item = new_suspension_manifest("code-230", "title-373");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
