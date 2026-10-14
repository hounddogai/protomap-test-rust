use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant score calculation.
pub trait TenantScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantScoreScoreOffsetAdjuster {
    pub weight: f64,
}

impl TenantScoreAdjuster for TenantScoreScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &TenantScore, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryTenantScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantScoreAdjuster for CountryTenantScoreAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &TenantScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantScoreAdjuster {
    pub minimum: f64,
}

impl TenantScoreAdjuster for MinimumTenantScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantScoreAdjuster {
    pub step: f64,
}

impl TenantScoreAdjuster for RoundingTenantScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tenant score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantScoreEngine {
    pub steps: Vec<Rc<dyn TenantScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantScoreEngine {
    pub fn compute(&mut self, item: &TenantScore) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TenantScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tenant_score_engine() -> TenantScoreEngine {
    TenantScoreEngine {
        steps: vec![
            Rc::new(TenantScoreScoreOffsetAdjuster { weight: 1.76 }) as Rc<dyn TenantScoreAdjuster>,
            Rc::new(CountryTenantScoreAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.96), ("west".to_string(), 1.25)]),
            }),
            Rc::new(MinimumTenantScoreAdjuster { minimum: 22.6 }),
            Rc::new(RoundingTenantScoreAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 35353.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_score_engine();
        let item = new_tenant_score("code-679", "title-612");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
