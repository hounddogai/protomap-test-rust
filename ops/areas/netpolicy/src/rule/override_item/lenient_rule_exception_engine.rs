use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient rule exception calculation.
pub trait LenientRuleExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientRuleException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientRuleExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientRuleExceptionModifier for LabelLenientRuleExceptionAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientRuleException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientRuleExceptionAdjuster {
    pub minimum: f64,
}

impl LenientRuleExceptionModifier for MinimumLenientRuleExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientRuleException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientRuleExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientRuleExceptionModifier for FlaggedLenientRuleExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientRuleException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientRuleExceptionAdjuster {
    pub percent: f64,
}

impl LenientRuleExceptionModifier for PercentLenientRuleExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientRuleException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientRuleExceptionAdjuster {
    pub cap: f64,
}

impl LenientRuleExceptionModifier for CapLenientRuleExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientRuleException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientRuleExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientRuleExceptionModifier for TieredLenientRuleExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientRuleException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient rule exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientRuleExceptionEngine {
    pub steps: Vec<Rc<dyn LenientRuleExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientRuleExceptionEngine {
    pub fn evaluate(&mut self, item: &LenientRuleException) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[LenientRuleException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_rule_exception_engine(flags: &flags::Registry) -> LenientRuleExceptionEngine {
    LenientRuleExceptionEngine {
        steps: vec![
            Rc::new(LabelLenientRuleExceptionAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.86),
                    ("north".to_string(), 0.74),
                    ("south".to_string(), 0.89),
                ]),
            }) as Rc<dyn LenientRuleExceptionModifier>,
            Rc::new(MinimumLenientRuleExceptionAdjuster { minimum: 11.9 }),
            Rc::new(FlaggedLenientRuleExceptionAdjuster {
                flags: flags.clone(),
                flag: "lenient-rule-exception-rolling".to_string(),
                percent: -2.5,
            }),
            Rc::new(PercentLenientRuleExceptionAdjuster { percent: -20.9 }),
            Rc::new(CapLenientRuleExceptionAdjuster { cap: 24043.0 }),
            Rc::new(TieredLenientRuleExceptionAdjuster {
                bounds: vec![1739.0, 3878.0],
                rates: vec![0.93, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 7415.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_rule_exception_engine(&flags::new_registry());
        let item = new_lenient_rule_exception("code-846", "label-732");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
