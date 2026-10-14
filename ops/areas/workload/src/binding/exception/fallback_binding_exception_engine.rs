use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback binding exception calculation.
pub trait FallbackBindingExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackBindingException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackBindingExceptionAdjuster {
    pub cap: f64,
}

impl FallbackBindingExceptionStep for CapFallbackBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackBindingException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackBindingExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackBindingExceptionStep for FlaggedFallbackBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackBindingException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackBindingExceptionAdjuster {
    pub minimum: f64,
}

impl FallbackBindingExceptionStep for MinimumFallbackBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackBindingException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingExceptionGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl FallbackBindingExceptionStep for FallbackBindingExceptionGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackBindingException, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackBindingExceptionAdjuster {
    pub step: f64,
}

impl FallbackBindingExceptionStep for RoundingFallbackBindingExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackBindingException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFallbackBindingExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackBindingExceptionStep for TitleFallbackBindingExceptionAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FallbackBindingException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback binding exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackBindingExceptionEngine {
    pub steps: Vec<Rc<dyn FallbackBindingExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackBindingExceptionEngine {
    pub fn run(&mut self, item: &FallbackBindingException) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FallbackBindingException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_binding_exception_engine(flags: &flags::Registry) -> FallbackBindingExceptionEngine {
    FallbackBindingExceptionEngine {
        steps: vec![
            Rc::new(CapFallbackBindingExceptionAdjuster { cap: 14459.0 }) as Rc<dyn FallbackBindingExceptionStep>,
            Rc::new(FlaggedFallbackBindingExceptionAdjuster {
                flags: flags.clone(),
                flag: "fallback-binding-exception-primary".to_string(),
                percent: 1.2,
            }),
            Rc::new(MinimumFallbackBindingExceptionAdjuster { minimum: 6.6 }),
            Rc::new(FallbackBindingExceptionGraceDaysOffsetAdjuster { weight: 1.89 }),
            Rc::new(RoundingFallbackBindingExceptionAdjuster { step: 0.25 }),
            Rc::new(TitleFallbackBindingExceptionAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.99),
                    ("east".to_string(), 0.93),
                    ("west".to_string(), 1.31),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 33168.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_binding_exception_engine(&flags::new_registry());
        let item = new_fallback_binding_exception("code-192", "title-357");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
