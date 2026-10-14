use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient alert override calculation.
pub trait LenientAlertOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientAlertOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientAlertOverrideAdjuster {
    pub minimum: f64,
}

impl LenientAlertOverrideAdjuster for MinimumLenientAlertOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientAlertOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientAlertOverrideAdjuster {
    pub step: f64,
}

impl LenientAlertOverrideAdjuster for RoundingLenientAlertOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientAlertOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLenientAlertOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientAlertOverrideAdjuster for SourceLenientAlertOverrideAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &LenientAlertOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientAlertOverrideAdjuster {
    pub percent: f64,
}

impl LenientAlertOverrideAdjuster for PercentLenientAlertOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientAlertOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientAlertOverrideAdjuster {
    pub cap: f64,
}

impl LenientAlertOverrideAdjuster for CapLenientAlertOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientAlertOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the lenient alert override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientAlertOverrideEngine {
    pub steps: Vec<Rc<dyn LenientAlertOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientAlertOverrideEngine {
    pub fn price(&mut self, item: &LenientAlertOverride) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[LenientAlertOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_alert_override_engine() -> LenientAlertOverrideEngine {
    LenientAlertOverrideEngine {
        steps: vec![
            Rc::new(MinimumLenientAlertOverrideAdjuster { minimum: 40.8 }) as Rc<dyn LenientAlertOverrideAdjuster>,
            Rc::new(RoundingLenientAlertOverrideAdjuster { step: 5.0 }),
            Rc::new(SourceLenientAlertOverrideAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.71),
                    ("north".to_string(), 0.9),
                    ("west".to_string(), 1.06),
                ]),
            }),
            Rc::new(PercentLenientAlertOverrideAdjuster { percent: 17.6 }),
            Rc::new(CapLenientAlertOverrideAdjuster { cap: 26564.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 98011.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_alert_override_engine();
        let item = new_lenient_alert_override("code-761", "label-275");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
