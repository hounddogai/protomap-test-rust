use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity target calculation.
pub trait IdentityTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleIdentityTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityTargetFactor for LocaleIdentityTargetAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &IdentityTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityTargetFactor for TieredIdentityTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityTargetAdjuster {
    pub cap: f64,
}

impl IdentityTargetFactor for CapIdentityTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTargetMarginOffsetAdjuster {
    pub weight: f64,
}

impl IdentityTargetFactor for IdentityTargetMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &IdentityTarget, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the identity target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityTargetEngine {
    pub steps: Vec<Rc<dyn IdentityTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityTargetEngine {
    pub fn run(&mut self, item: &IdentityTarget) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[IdentityTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_identity_target_engine() -> IdentityTargetEngine {
    IdentityTargetEngine {
        steps: vec![
            Rc::new(LocaleIdentityTargetAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.16),
                    ("north".to_string(), 0.75),
                    ("central".to_string(), 1.22),
                ]),
            }) as Rc<dyn IdentityTargetFactor>,
            Rc::new(TieredIdentityTargetAdjuster {
                bounds: vec![2114.0, 3426.0],
                rates: vec![0.89, 1.14],
            }),
            Rc::new(CapIdentityTargetAdjuster { cap: 27000.0 }),
            Rc::new(IdentityTargetMarginOffsetAdjuster { weight: 0.66 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 93223.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_target_engine();
        let item = new_identity_target("code-791", "title-329");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
