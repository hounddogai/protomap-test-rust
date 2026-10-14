use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict binding exception calculation.
pub trait StrictBindingExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictBindingException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingExceptionGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl StrictBindingExceptionAdjuster for StrictBindingExceptionGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &StrictBindingException, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictBindingExceptionAdjuster {
    pub percent: f64,
}

impl StrictBindingExceptionAdjuster for PercentStrictBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictBindingException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictBindingExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictBindingExceptionAdjuster for TieredStrictBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictBindingException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictBindingExceptionAdjuster {
    pub minimum: f64,
}

impl StrictBindingExceptionAdjuster for MinimumStrictBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictBindingException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictBindingExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictBindingExceptionAdjuster for FlaggedStrictBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictBindingException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict binding exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictBindingExceptionEngine {
    pub steps: Vec<Rc<dyn StrictBindingExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictBindingExceptionEngine {
    pub fn evaluate(&mut self, item: &StrictBindingException) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictBindingExceptionAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictBindingException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_binding_exception_engine(flags: &flags::Registry) -> StrictBindingExceptionEngine {
    StrictBindingExceptionEngine {
        steps: vec![
            Rc::new(StrictBindingExceptionGraceDaysOffsetAdjuster { weight: 0.71 })
                as Rc<dyn StrictBindingExceptionAdjuster>,
            Rc::new(PercentStrictBindingExceptionAdjuster { percent: -16.8 }),
            Rc::new(TieredStrictBindingExceptionAdjuster {
                bounds: vec![8785.0, 9462.0],
                rates: vec![1.18, 0.84],
            }),
            Rc::new(MinimumStrictBindingExceptionAdjuster { minimum: 12.0 }),
            Rc::new(FlaggedStrictBindingExceptionAdjuster {
                flags: flags.clone(),
                flag: "strict-binding-exception-tiered".to_string(),
                percent: 9.9,
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 11662.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_binding_exception_engine(&flags::new_registry());
        let item = new_strict_binding_exception("code-925", "name-770");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
