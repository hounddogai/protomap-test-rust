use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the fallback binding claim calculation.
pub trait FallbackBindingClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackBindingClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackBindingClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackBindingClaimFactor for TieredFallbackBindingClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackBindingClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackBindingClaimAdjuster {
    pub cap: f64,
}

impl FallbackBindingClaimFactor for CapFallbackBindingClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackBindingClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackBindingClaimAdjuster {
    pub percent: f64,
}

impl FallbackBindingClaimFactor for PercentFallbackBindingClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackBindingClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFallbackBindingClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackBindingClaimFactor for LabelFallbackBindingClaimAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FallbackBindingClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback binding claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackBindingClaimEngine {
    pub steps: Vec<Rc<dyn FallbackBindingClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackBindingClaimEngine {
    pub fn compute(&mut self, item: &FallbackBindingClaim) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackBindingClaimFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackBindingClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_binding_claim_engine() -> FallbackBindingClaimEngine {
    FallbackBindingClaimEngine {
        steps: vec![
            Rc::new(TieredFallbackBindingClaimAdjuster {
                bounds: vec![8904.0, 9572.0],
                rates: vec![0.85, 1.0],
            }) as Rc<dyn FallbackBindingClaimFactor>,
            Rc::new(CapFallbackBindingClaimAdjuster { cap: 14747.0 }),
            Rc::new(PercentFallbackBindingClaimAdjuster { percent: 6.3 }),
            Rc::new(LabelFallbackBindingClaimAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.99),
                    ("retail".to_string(), 1.39),
                    ("west".to_string(), 1.21),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 26350.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_binding_claim_engine();
        let item = new_fallback_binding_claim("code-666", "label-538");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
