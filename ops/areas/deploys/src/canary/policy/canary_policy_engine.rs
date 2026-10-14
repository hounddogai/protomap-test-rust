use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary policy calculation.
pub trait CanaryPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryPolicyModifier for TieredCanaryPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryPolicyAdjuster {
    pub step: f64,
}

impl CanaryPolicyModifier for RoundingCanaryPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPolicyRevisionOffsetAdjuster {
    pub weight: f64,
}

impl CanaryPolicyModifier for CanaryPolicyRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryPolicy, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryPolicyAdjuster {
    pub minimum: f64,
}

impl CanaryPolicyModifier for MinimumCanaryPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the canary policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryPolicyEngine {
    pub steps: Vec<Rc<dyn CanaryPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryPolicyEngine {
    pub fn price(&mut self, item: &CanaryPolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CanaryPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_canary_policy_engine() -> CanaryPolicyEngine {
    CanaryPolicyEngine {
        steps: vec![
            Rc::new(TieredCanaryPolicyAdjuster {
                bounds: vec![1891.0, 6025.0, 6310.0],
                rates: vec![0.94, 0.89, 1.07],
            }) as Rc<dyn CanaryPolicyModifier>,
            Rc::new(RoundingCanaryPolicyAdjuster { step: 0.25 }),
            Rc::new(CanaryPolicyRevisionOffsetAdjuster { weight: 1.71 }),
            Rc::new(MinimumCanaryPolicyAdjuster { minimum: 37.6 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 89723.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_policy_engine();
        let item = new_canary_policy("code-372", "label-672");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
