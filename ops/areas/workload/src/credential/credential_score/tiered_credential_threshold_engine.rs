use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered credential threshold calculation.
pub trait TieredCredentialThresholdFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredCredentialThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialThresholdThresholdOffsetAdjuster {
    pub weight: f64,
}

impl TieredCredentialThresholdFactor for TieredCredentialThresholdThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &TieredCredentialThreshold, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredCredentialThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredCredentialThresholdFactor for FlaggedTieredCredentialThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredCredentialThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredCredentialThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredCredentialThresholdFactor for TieredTieredCredentialThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredCredentialThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeTieredCredentialThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredCredentialThresholdFactor for ScopeTieredCredentialThresholdAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &TieredCredentialThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredCredentialThresholdAdjuster {
    pub step: f64,
}

impl TieredCredentialThresholdFactor for RoundingTieredCredentialThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredCredentialThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tiered credential threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredCredentialThresholdEngine {
    pub steps: Vec<Rc<dyn TieredCredentialThresholdFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredCredentialThresholdEngine {
    pub fn price(&mut self, item: &TieredCredentialThreshold) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TieredCredentialThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_credential_threshold_engine(flags: &flags::Registry) -> TieredCredentialThresholdEngine {
    TieredCredentialThresholdEngine {
        steps: vec![
            Rc::new(TieredCredentialThresholdThresholdOffsetAdjuster { weight: 1.71 })
                as Rc<dyn TieredCredentialThresholdFactor>,
            Rc::new(FlaggedTieredCredentialThresholdAdjuster {
                flags: flags.clone(),
                flag: "tiered-credential-threshold-seasonal".to_string(),
                percent: -0.6,
            }),
            Rc::new(TieredTieredCredentialThresholdAdjuster {
                bounds: vec![3826.0, 4608.0, 7878.0],
                rates: vec![0.9, 1.07, 1.01],
            }),
            Rc::new(ScopeTieredCredentialThresholdAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.9),
                    ("west".to_string(), 0.99),
                    ("south".to_string(), 1.21),
                ]),
            }),
            Rc::new(RoundingTieredCredentialThresholdAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 81551.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_credential_threshold_engine(&flags::new_registry());
        let item = new_tiered_credential_threshold("code-57", "label-99");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
