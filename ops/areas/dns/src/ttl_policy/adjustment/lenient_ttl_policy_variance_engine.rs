use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the lenient ttl policy variance calculation.
pub trait LenientTtlPolicyVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientTtlPolicyVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyVarianceRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl LenientTtlPolicyVarianceModifier for LenientTtlPolicyVarianceRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &LenientTtlPolicyVariance, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientTtlPolicyVarianceAdjuster {
    pub cap: f64,
}

impl LenientTtlPolicyVarianceModifier for CapLenientTtlPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientTtlPolicyVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryLenientTtlPolicyVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientTtlPolicyVarianceModifier for CountryLenientTtlPolicyVarianceAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &LenientTtlPolicyVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient ttl policy variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientTtlPolicyVarianceEngine {
    pub steps: Vec<Rc<dyn LenientTtlPolicyVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientTtlPolicyVarianceEngine {
    pub fn price(&mut self, item: &LenientTtlPolicyVariance) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientTtlPolicyVarianceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientTtlPolicyVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_ttl_policy_variance_engine() -> LenientTtlPolicyVarianceEngine {
    LenientTtlPolicyVarianceEngine {
        steps: vec![
            Rc::new(LenientTtlPolicyVarianceRotationDaysOffsetAdjuster { weight: 1.79 })
                as Rc<dyn LenientTtlPolicyVarianceModifier>,
            Rc::new(CapLenientTtlPolicyVarianceAdjuster { cap: 30059.0 }),
            Rc::new(CountryLenientTtlPolicyVarianceAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.19), ("west".to_string(), 0.84)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 56015.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_ttl_policy_variance_engine();
        let item = new_lenient_ttl_policy_variance("code-30", "name-868");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
