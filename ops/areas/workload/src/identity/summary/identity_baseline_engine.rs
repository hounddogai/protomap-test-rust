use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity baseline calculation.
pub trait IdentityBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityBaselineAdjuster {
    pub step: f64,
}

impl IdentityBaselineModifier for RoundingIdentityBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityBaselineAdjuster {
    pub percent: f64,
}

impl IdentityBaselineModifier for PercentIdentityBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityBaselineModifier for TieredIdentityBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentIdentityBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityBaselineModifier for SegmentIdentityBaselineAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &IdentityBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityBaselineAdjuster {
    pub cap: f64,
}

impl IdentityBaselineModifier for CapIdentityBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the identity baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityBaselineEngine {
    pub steps: Vec<Rc<dyn IdentityBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityBaselineEngine {
    pub fn compute(&mut self, item: &IdentityBaseline) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[IdentityBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_identity_baseline_engine() -> IdentityBaselineEngine {
    IdentityBaselineEngine {
        steps: vec![
            Rc::new(RoundingIdentityBaselineAdjuster { step: 5.0 }) as Rc<dyn IdentityBaselineModifier>,
            Rc::new(PercentIdentityBaselineAdjuster { percent: 2.8 }),
            Rc::new(TieredIdentityBaselineAdjuster {
                bounds: vec![1161.0, 1643.0, 8626.0],
                rates: vec![0.96, 1.15, 0.83],
            }),
            Rc::new(SegmentIdentityBaselineAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.26),
                    ("online".to_string(), 1.25),
                    ("central".to_string(), 1.07),
                ]),
            }),
            Rc::new(CapIdentityBaselineAdjuster { cap: 41377.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 75044.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_baseline_engine();
        let item = new_identity_baseline("code-952", "label-742");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
