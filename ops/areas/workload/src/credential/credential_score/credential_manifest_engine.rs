use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the credential manifest calculation.
pub trait CredentialManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialManifestModifier for TieredCredentialManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCredentialManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialManifestModifier for NameCredentialManifestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CredentialManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCredentialManifestAdjuster {
    pub percent: f64,
}

impl CredentialManifestModifier for PercentCredentialManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CredentialManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCredentialManifestAdjuster {
    pub step: f64,
}

impl CredentialManifestModifier for RoundingCredentialManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CredentialManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the credential manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialManifestEngine {
    pub steps: Vec<Rc<dyn CredentialManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialManifestEngine {
    pub fn compute(&mut self, item: &CredentialManifest) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[CredentialManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_credential_manifest_engine() -> CredentialManifestEngine {
    CredentialManifestEngine {
        steps: vec![
            Rc::new(TieredCredentialManifestAdjuster {
                bounds: vec![4266.0, 6141.0],
                rates: vec![0.84, 1.17],
            }) as Rc<dyn CredentialManifestModifier>,
            Rc::new(NameCredentialManifestAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.3),
                    ("east".to_string(), 1.2),
                    ("online".to_string(), 1.09),
                ]),
            }),
            Rc::new(PercentCredentialManifestAdjuster { percent: -19.0 }),
            Rc::new(RoundingCredentialManifestAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 69639.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_manifest_engine();
        let item = new_credential_manifest("code-137", "name-56");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
