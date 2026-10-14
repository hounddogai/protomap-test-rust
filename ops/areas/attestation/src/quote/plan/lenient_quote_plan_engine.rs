use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient quote plan calculation.
pub trait LenientQuotePlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientQuotePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientQuotePlanAdjuster {
    pub minimum: f64,
}

impl LenientQuotePlanFactor for MinimumLenientQuotePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientQuotePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLenientQuotePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientQuotePlanFactor for TitleLenientQuotePlanAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LenientQuotePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientQuotePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientQuotePlanFactor for TieredLenientQuotePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientQuotePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientQuotePlanAdjuster {
    pub step: f64,
}

impl LenientQuotePlanFactor for RoundingLenientQuotePlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientQuotePlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient quote plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientQuotePlanEngine {
    pub steps: Vec<Rc<dyn LenientQuotePlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientQuotePlanEngine {
    pub fn compute(&mut self, item: &LenientQuotePlan) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientQuotePlanFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientQuotePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_quote_plan_engine() -> LenientQuotePlanEngine {
    LenientQuotePlanEngine {
        steps: vec![
            Rc::new(MinimumLenientQuotePlanAdjuster { minimum: 0.1 }) as Rc<dyn LenientQuotePlanFactor>,
            Rc::new(TitleLenientQuotePlanAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.85), ("east".to_string(), 0.85)]),
            }),
            Rc::new(TieredLenientQuotePlanAdjuster {
                bounds: vec![350.0, 4411.0, 8376.0],
                rates: vec![0.98, 0.96, 1.16],
            }),
            Rc::new(RoundingLenientQuotePlanAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 3013.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_quote_plan_engine();
        let item = new_lenient_quote_plan("code-647", "title-923");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
