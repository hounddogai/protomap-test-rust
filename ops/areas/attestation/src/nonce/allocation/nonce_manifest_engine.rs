use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the nonce manifest calculation.
pub trait NonceManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceManifestFactor for TieredNonceManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceManifestAdjuster {
    pub cap: f64,
}

impl NonceManifestFactor for CapNonceManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceNonceManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl NonceManifestFactor for NamespaceNonceManifestAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &NonceManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNonceManifestAdjuster {
    pub minimum: f64,
}

impl NonceManifestFactor for MinimumNonceManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NonceManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the nonce manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceManifestEngine {
    pub steps: Vec<Rc<dyn NonceManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceManifestEngine {
    pub fn run(&mut self, item: &NonceManifest) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[NonceManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_nonce_manifest_engine() -> NonceManifestEngine {
    NonceManifestEngine {
        steps: vec![
            Rc::new(TieredNonceManifestAdjuster {
                bounds: vec![36.0, 5799.0, 7238.0],
                rates: vec![0.84, 0.93, 0.95],
            }) as Rc<dyn NonceManifestFactor>,
            Rc::new(CapNonceManifestAdjuster { cap: 47509.0 }),
            Rc::new(NamespaceNonceManifestAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.31),
                    ("central".to_string(), 0.92),
                    ("south".to_string(), 0.93),
                ]),
            }),
            Rc::new(MinimumNonceManifestAdjuster { minimum: 27.5 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 63974.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_manifest_engine();
        let item = new_nonce_manifest("code-660", "title-537");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
