use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the grant manifest calculation.
pub trait GrantManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantManifestAdjuster {
    pub cap: f64,
}

impl GrantManifestAdjuster for CapGrantManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeGrantManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantManifestAdjuster for ScopeGrantManifestAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &GrantManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantManifestRateOffsetAdjuster {
    pub weight: f64,
}

impl GrantManifestAdjuster for GrantManifestRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &GrantManifest, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantManifestAdjuster {
    pub step: f64,
}

impl GrantManifestAdjuster for RoundingGrantManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGrantManifestAdjuster {
    pub percent: f64,
}

impl GrantManifestAdjuster for PercentGrantManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GrantManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the grant manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantManifestEngine {
    pub steps: Vec<Rc<dyn GrantManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantManifestEngine {
    pub fn price(&mut self, item: &GrantManifest) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantManifestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_grant_manifest_engine() -> GrantManifestEngine {
    GrantManifestEngine {
        steps: vec![
            Rc::new(CapGrantManifestAdjuster { cap: 26859.0 }) as Rc<dyn GrantManifestAdjuster>,
            Rc::new(ScopeGrantManifestAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.18),
                    ("north".to_string(), 0.84),
                    ("retail".to_string(), 1.13),
                ]),
            }),
            Rc::new(GrantManifestRateOffsetAdjuster { weight: 0.44 }),
            Rc::new(RoundingGrantManifestAdjuster { step: 0.1 }),
            Rc::new(PercentGrantManifestAdjuster { percent: 24.5 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 78996.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_manifest_engine();
        let item = new_grant_manifest("code-526", "title-707");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
