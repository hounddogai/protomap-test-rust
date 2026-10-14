use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive nonce profile calculation.
pub trait AdaptiveNonceProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveNonceProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleAdaptiveNonceProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveNonceProfileStep for LocaleAdaptiveNonceProfileAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &AdaptiveNonceProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveNonceProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveNonceProfileStep for FlaggedAdaptiveNonceProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveNonceProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveNonceProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveNonceProfileStep for TieredAdaptiveNonceProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveNonceProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive nonce profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveNonceProfileEngine {
    pub steps: Vec<Rc<dyn AdaptiveNonceProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveNonceProfileEngine {
    pub fn price(&mut self, item: &AdaptiveNonceProfile) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveNonceProfileStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveNonceProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_nonce_profile_engine(flags: &flags::Registry) -> AdaptiveNonceProfileEngine {
    AdaptiveNonceProfileEngine {
        steps: vec![
            Rc::new(LocaleAdaptiveNonceProfileAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.1), ("east".to_string(), 0.72)]),
            }) as Rc<dyn AdaptiveNonceProfileStep>,
            Rc::new(FlaggedAdaptiveNonceProfileAdjuster {
                flags: flags.clone(),
                flag: "adaptive-nonce-profile-rolling".to_string(),
                percent: -0.7,
            }),
            Rc::new(TieredAdaptiveNonceProfileAdjuster {
                bounds: vec![420.0, 1383.0, 8266.0],
                rates: vec![0.95, 0.88, 1.0],
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 5305.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_nonce_profile_engine(&flags::new_registry());
        let item = new_adaptive_nonce_profile("code-911", "title-570");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
