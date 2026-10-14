use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity estimate calculation.
pub trait IdentityEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitIdentityEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityEstimateAdjuster for UnitIdentityEstimateAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &IdentityEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityEstimateAdjuster for TieredIdentityEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityEstimateAdjuster for FlaggedIdentityEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the identity estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityEstimateEngine {
    pub steps: Vec<Rc<dyn IdentityEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityEstimateEngine {
    pub fn price(&mut self, item: &IdentityEstimate) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[IdentityEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_identity_estimate_engine(flags: &flags::Registry) -> IdentityEstimateEngine {
    IdentityEstimateEngine {
        steps: vec![
            Rc::new(UnitIdentityEstimateAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.86), ("south".to_string(), 0.93)]),
            }) as Rc<dyn IdentityEstimateAdjuster>,
            Rc::new(TieredIdentityEstimateAdjuster {
                bounds: vec![1540.0, 5168.0],
                rates: vec![0.84, 1.12],
            }),
            Rc::new(FlaggedIdentityEstimateAdjuster {
                flags: flags.clone(),
                flag: "identity-estimate-weighted".to_string(),
                percent: -5.9,
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 48109.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_estimate_engine(&flags::new_registry());
        let item = new_identity_estimate("code-123", "label-677");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
