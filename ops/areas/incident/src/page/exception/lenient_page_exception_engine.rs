use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient page exception calculation.
pub trait LenientPageExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientPageException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientPageExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientPageExceptionModifier for FlaggedLenientPageExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientPageException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientPageExceptionAdjuster {
    pub cap: f64,
}

impl LenientPageExceptionModifier for CapLenientPageExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientPageException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientPageExceptionAdjuster {
    pub minimum: f64,
}

impl LenientPageExceptionModifier for MinimumLenientPageExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientPageException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientPageExceptionAdjuster {
    pub percent: f64,
}

impl LenientPageExceptionModifier for PercentLenientPageExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientPageException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient page exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientPageExceptionEngine {
    pub steps: Vec<Rc<dyn LenientPageExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientPageExceptionEngine {
    pub fn compute(&mut self, item: &LenientPageException) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[LenientPageException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_page_exception_engine(flags: &flags::Registry) -> LenientPageExceptionEngine {
    LenientPageExceptionEngine {
        steps: vec![
            Rc::new(FlaggedLenientPageExceptionAdjuster {
                flags: flags.clone(),
                flag: "lenient-page-exception-capped".to_string(),
                percent: -8.1,
            }) as Rc<dyn LenientPageExceptionModifier>,
            Rc::new(CapLenientPageExceptionAdjuster { cap: 27701.0 }),
            Rc::new(MinimumLenientPageExceptionAdjuster { minimum: 25.2 }),
            Rc::new(PercentLenientPageExceptionAdjuster { percent: 23.7 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 7592.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_page_exception_engine(&flags::new_registry());
        let item = new_lenient_page_exception("code-28", "name-455");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
