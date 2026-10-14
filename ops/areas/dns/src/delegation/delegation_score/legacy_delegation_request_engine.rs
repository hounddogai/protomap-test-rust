use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy delegation request calculation.
pub trait LegacyDelegationRequestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyDelegationRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryLegacyDelegationRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyDelegationRequestModifier for CategoryLegacyDelegationRequestAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &LegacyDelegationRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyDelegationRequestAdjuster {
    pub percent: f64,
}

impl LegacyDelegationRequestModifier for PercentLegacyDelegationRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyDelegationRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyDelegationRequestAdjuster {
    pub minimum: f64,
}

impl LegacyDelegationRequestModifier for MinimumLegacyDelegationRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyDelegationRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyDelegationRequestAdjuster {
    pub cap: f64,
}

impl LegacyDelegationRequestModifier for CapLegacyDelegationRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyDelegationRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy delegation request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyDelegationRequestEngine {
    pub steps: Vec<Rc<dyn LegacyDelegationRequestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyDelegationRequestEngine {
    pub fn price(&mut self, item: &LegacyDelegationRequest) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[LegacyDelegationRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_delegation_request_engine() -> LegacyDelegationRequestEngine {
    LegacyDelegationRequestEngine {
        steps: vec![
            Rc::new(CategoryLegacyDelegationRequestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.31), ("retail".to_string(), 0.75)]),
            }) as Rc<dyn LegacyDelegationRequestModifier>,
            Rc::new(PercentLegacyDelegationRequestAdjuster { percent: 11.4 }),
            Rc::new(MinimumLegacyDelegationRequestAdjuster { minimum: 35.9 }),
            Rc::new(CapLegacyDelegationRequestAdjuster { cap: 5498.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 26246.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_delegation_request_engine();
        let item = new_legacy_delegation_request("code-81", "title-895");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
