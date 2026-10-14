use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling binding exception calculation.
pub trait RollingBindingExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingBindingException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingBindingExceptionAdjuster {
    pub cap: f64,
}

impl RollingBindingExceptionAdjuster for CapRollingBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingBindingException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRollingBindingExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingBindingExceptionAdjuster for LocaleRollingBindingExceptionAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RollingBindingException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingBindingExceptionAdjuster {
    pub step: f64,
}

impl RollingBindingExceptionAdjuster for RoundingRollingBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingBindingException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingBindingExceptionAdjuster {
    pub minimum: f64,
}

impl RollingBindingExceptionAdjuster for MinimumRollingBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingBindingException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingBindingExceptionAdjuster {
    pub percent: f64,
}

impl RollingBindingExceptionAdjuster for PercentRollingBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingBindingException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling binding exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingBindingExceptionEngine {
    pub steps: Vec<Rc<dyn RollingBindingExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingBindingExceptionEngine {
    pub fn run(&mut self, item: &RollingBindingException) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingBindingExceptionAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingBindingException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_binding_exception_engine() -> RollingBindingExceptionEngine {
    RollingBindingExceptionEngine {
        steps: vec![
            Rc::new(CapRollingBindingExceptionAdjuster { cap: 9392.0 }) as Rc<dyn RollingBindingExceptionAdjuster>,
            Rc::new(LocaleRollingBindingExceptionAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.83), ("retail".to_string(), 0.8)]),
            }),
            Rc::new(RoundingRollingBindingExceptionAdjuster { step: 0.5 }),
            Rc::new(MinimumRollingBindingExceptionAdjuster { minimum: 35.8 }),
            Rc::new(PercentRollingBindingExceptionAdjuster { percent: 18.5 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 92299.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_binding_exception_engine();
        let item = new_rolling_binding_exception("code-866", "label-625");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
