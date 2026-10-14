use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the ttl policy override calculation.
pub trait TtlPolicyOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyOverrideAdjuster for TieredTtlPolicyOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyOverrideAdjuster {
    pub percent: f64,
}

impl TtlPolicyOverrideAdjuster for PercentTtlPolicyOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryTtlPolicyOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyOverrideAdjuster for CountryTtlPolicyOverrideAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &TtlPolicyOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyOverrideAdjuster {
    pub cap: f64,
}

impl TtlPolicyOverrideAdjuster for CapTtlPolicyOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the ttl policy override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyOverrideEngine {
    pub steps: Vec<Rc<dyn TtlPolicyOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyOverrideEngine {
    pub fn evaluate(&mut self, item: &TtlPolicyOverride) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_ttl_policy_override_engine() -> TtlPolicyOverrideEngine {
    TtlPolicyOverrideEngine {
        steps: vec![
            Rc::new(TieredTtlPolicyOverrideAdjuster {
                bounds: vec![2517.0, 6772.0, 7181.0, 8163.0],
                rates: vec![0.93, 0.98, 1.1, 1.04],
            }) as Rc<dyn TtlPolicyOverrideAdjuster>,
            Rc::new(PercentTtlPolicyOverrideAdjuster { percent: 10.2 }),
            Rc::new(CountryTtlPolicyOverrideAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.01), ("south".to_string(), 1.11)]),
            }),
            Rc::new(CapTtlPolicyOverrideAdjuster { cap: 4010.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 33788.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_override_engine();
        let item = new_ttl_policy_override("code-341", "title-585");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
