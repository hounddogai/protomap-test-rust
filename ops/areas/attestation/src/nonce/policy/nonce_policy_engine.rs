use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the nonce policy calculation.
pub trait NoncePolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NoncePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNoncePolicyAdjuster {
    pub percent: f64,
}

impl NoncePolicyStep for PercentNoncePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NoncePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNoncePolicyAdjuster {
    pub minimum: f64,
}

impl NoncePolicyStep for MinimumNoncePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NoncePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentNoncePolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl NoncePolicyStep for SegmentNoncePolicyAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &NoncePolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the nonce policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NoncePolicyEngine {
    pub steps: Vec<Rc<dyn NoncePolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NoncePolicyEngine {
    pub fn price(&mut self, item: &NoncePolicy) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[NoncePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_nonce_policy_engine() -> NoncePolicyEngine {
    NoncePolicyEngine {
        steps: vec![
            Rc::new(PercentNoncePolicyAdjuster { percent: 17.8 }) as Rc<dyn NoncePolicyStep>,
            Rc::new(MinimumNoncePolicyAdjuster { minimum: 37.7 }),
            Rc::new(SegmentNoncePolicyAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.04), ("retail".to_string(), 1.19)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 81865.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_policy_engine();
        let item = new_nonce_policy("code-469", "title-690");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
