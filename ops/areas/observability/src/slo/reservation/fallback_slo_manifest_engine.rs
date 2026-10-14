use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback slo manifest calculation.
pub trait FallbackSloManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackSloManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFallbackSloManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackSloManifestAdjuster for NamespaceFallbackSloManifestAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FallbackSloManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackSloManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackSloManifestAdjuster for TieredFallbackSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackSloManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackSloManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackSloManifestAdjuster for FlaggedFallbackSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackSloManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackSloManifestAdjuster {
    pub minimum: f64,
}

impl FallbackSloManifestAdjuster for MinimumFallbackSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackSloManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackSloManifestAdjuster {
    pub percent: f64,
}

impl FallbackSloManifestAdjuster for PercentFallbackSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackSloManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackSloManifestAdjuster {
    pub cap: f64,
}

impl FallbackSloManifestAdjuster for CapFallbackSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackSloManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback slo manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackSloManifestEngine {
    pub steps: Vec<Rc<dyn FallbackSloManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackSloManifestEngine {
    pub fn price(&mut self, item: &FallbackSloManifest) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackSloManifestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackSloManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_slo_manifest_engine(flags: &flags::Registry) -> FallbackSloManifestEngine {
    FallbackSloManifestEngine {
        steps: vec![
            Rc::new(NamespaceFallbackSloManifestAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.85)]),
            }) as Rc<dyn FallbackSloManifestAdjuster>,
            Rc::new(TieredFallbackSloManifestAdjuster {
                bounds: vec![8434.0, 9071.0],
                rates: vec![1.18, 1.19],
            }),
            Rc::new(FlaggedFallbackSloManifestAdjuster {
                flags: flags.clone(),
                flag: "fallback-slo-manifest-default".to_string(),
                percent: 3.1,
            }),
            Rc::new(MinimumFallbackSloManifestAdjuster { minimum: 41.1 }),
            Rc::new(PercentFallbackSloManifestAdjuster { percent: 6.2 }),
            Rc::new(CapFallbackSloManifestAdjuster { cap: 43986.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 53604.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_slo_manifest_engine(&flags::new_registry());
        let item = new_fallback_slo_manifest("code-215", "name-254");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
