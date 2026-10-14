use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the identity manifest calculation.
pub trait IdentityManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityManifestAdjuster {
    pub percent: f64,
}

impl IdentityManifestFactor for PercentIdentityManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterIdentityManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityManifestFactor for ClusterIdentityManifestAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &IdentityManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityManifestAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl IdentityManifestFactor for IdentityManifestAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &IdentityManifest, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityManifestAdjuster {
    pub cap: f64,
}

impl IdentityManifestFactor for CapIdentityManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the identity manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityManifestEngine {
    pub steps: Vec<Rc<dyn IdentityManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityManifestEngine {
    pub fn compute(&mut self, item: &IdentityManifest) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityManifestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_identity_manifest_engine() -> IdentityManifestEngine {
    IdentityManifestEngine {
        steps: vec![
            Rc::new(PercentIdentityManifestAdjuster { percent: -22.5 }) as Rc<dyn IdentityManifestFactor>,
            Rc::new(ClusterIdentityManifestAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.0), ("east".to_string(), 1.2)]),
            }),
            Rc::new(IdentityManifestAttemptsOffsetAdjuster { weight: 1.8 }),
            Rc::new(CapIdentityManifestAdjuster { cap: 32568.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 22354.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_manifest_engine();
        let item = new_identity_manifest("code-863", "title-323");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
