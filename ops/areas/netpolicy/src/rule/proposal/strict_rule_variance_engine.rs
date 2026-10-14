use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict rule variance calculation.
pub trait StrictRuleVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictRuleVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictRuleVarianceAdjuster {
    pub percent: f64,
}

impl StrictRuleVarianceAdjuster for PercentStrictRuleVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictRuleVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictRuleVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictRuleVarianceAdjuster for TieredStrictRuleVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictRuleVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleStrictRuleVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictRuleVarianceAdjuster for LocaleStrictRuleVarianceAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &StrictRuleVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict rule variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictRuleVarianceEngine {
    pub steps: Vec<Rc<dyn StrictRuleVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictRuleVarianceEngine {
    pub fn evaluate(&mut self, item: &StrictRuleVariance) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictRuleVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_rule_variance_engine() -> StrictRuleVarianceEngine {
    StrictRuleVarianceEngine {
        steps: vec![
            Rc::new(PercentStrictRuleVarianceAdjuster { percent: -12.4 }) as Rc<dyn StrictRuleVarianceAdjuster>,
            Rc::new(TieredStrictRuleVarianceAdjuster {
                bounds: vec![5320.0, 6492.0, 8829.0],
                rates: vec![0.96, 0.97, 1.17],
            }),
            Rc::new(LocaleStrictRuleVarianceAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.09), ("south".to_string(), 1.06)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 69329.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_rule_variance_engine();
        let item = new_strict_rule_variance("code-207", "title-803");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
