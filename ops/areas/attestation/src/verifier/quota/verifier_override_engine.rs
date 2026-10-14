use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the verifier override calculation.
pub trait VerifierOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierOverrideDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl VerifierOverrideModifier for VerifierOverrideDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierOverride, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentVerifierOverrideAdjuster {
    pub percent: f64,
}

impl VerifierOverrideModifier for PercentVerifierOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &VerifierOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterVerifierOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl VerifierOverrideModifier for ClusterVerifierOverrideAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &VerifierOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierOverrideAdjuster {
    pub cap: f64,
}

impl VerifierOverrideModifier for CapVerifierOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the verifier override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierOverrideEngine {
    pub steps: Vec<Rc<dyn VerifierOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierOverrideEngine {
    pub fn price(&mut self, item: &VerifierOverride) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[VerifierOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_verifier_override_engine() -> VerifierOverrideEngine {
    VerifierOverrideEngine {
        steps: vec![
            Rc::new(VerifierOverrideDurationMinutesOffsetAdjuster { weight: 1.68 }) as Rc<dyn VerifierOverrideModifier>,
            Rc::new(PercentVerifierOverrideAdjuster { percent: 12.1 }),
            Rc::new(ClusterVerifierOverrideAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.02)]),
            }),
            Rc::new(CapVerifierOverrideAdjuster { cap: 8465.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 72270.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_override_engine();
        let item = new_verifier_override("code-804", "title-309");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
