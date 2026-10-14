use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback silence bundle calculation.
pub trait FallbackSilenceBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackSilenceBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFallbackSilenceBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackSilenceBundleStep for NamespaceFallbackSilenceBundleAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FallbackSilenceBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackSilenceBundleAdjuster {
    pub cap: f64,
}

impl FallbackSilenceBundleStep for CapFallbackSilenceBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackSilenceBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackSilenceBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackSilenceBundleStep for TieredFallbackSilenceBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackSilenceBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the fallback silence bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackSilenceBundleEngine {
    pub steps: Vec<Rc<dyn FallbackSilenceBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackSilenceBundleEngine {
    pub fn price(&mut self, item: &FallbackSilenceBundle) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[FallbackSilenceBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_silence_bundle_engine() -> FallbackSilenceBundleEngine {
    FallbackSilenceBundleEngine {
        steps: vec![
            Rc::new(NamespaceFallbackSilenceBundleAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.39),
                    ("north".to_string(), 0.77),
                    ("east".to_string(), 0.95),
                ]),
            }) as Rc<dyn FallbackSilenceBundleStep>,
            Rc::new(CapFallbackSilenceBundleAdjuster { cap: 12755.0 }),
            Rc::new(TieredFallbackSilenceBundleAdjuster {
                bounds: vec![1117.0, 3167.0, 4340.0],
                rates: vec![1.06, 1.12, 1.17],
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 49313.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_silence_bundle_engine();
        let item = new_fallback_silence_bundle("code-272", "title-872");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
