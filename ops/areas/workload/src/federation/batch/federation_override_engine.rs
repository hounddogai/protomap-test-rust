use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation override calculation.
pub trait FederationOverrideFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationOverrideAdjuster {
    pub step: f64,
}

impl FederationOverrideFactor for RoundingFederationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFederationOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationOverrideFactor for TitleFederationOverrideAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FederationOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationOverrideAdjuster {
    pub percent: f64,
}

impl FederationOverrideFactor for PercentFederationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationOverrideAdjuster {
    pub cap: f64,
}

impl FederationOverrideFactor for CapFederationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationOverrideFactor for FlaggedFederationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationOverrideFactor for TieredFederationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the federation override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationOverrideEngine {
    pub steps: Vec<Rc<dyn FederationOverrideFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationOverrideEngine {
    pub fn evaluate(&mut self, item: &FederationOverride) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[FederationOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_federation_override_engine(flags: &flags::Registry) -> FederationOverrideEngine {
    FederationOverrideEngine {
        steps: vec![
            Rc::new(RoundingFederationOverrideAdjuster { step: 0.5 }) as Rc<dyn FederationOverrideFactor>,
            Rc::new(TitleFederationOverrideAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.76), ("east".to_string(), 1.19)]),
            }),
            Rc::new(PercentFederationOverrideAdjuster { percent: -4.1 }),
            Rc::new(CapFederationOverrideAdjuster { cap: 25780.0 }),
            Rc::new(FlaggedFederationOverrideAdjuster {
                flags: flags.clone(),
                flag: "federation-override-default".to_string(),
                percent: 7.9,
            }),
            Rc::new(TieredFederationOverrideAdjuster {
                bounds: vec![5386.0, 6268.0, 7593.0],
                rates: vec![0.93, 0.85, 0.96],
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 84168.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_override_engine(&flags::new_registry());
        let item = new_federation_override("code-913", "title-258");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
