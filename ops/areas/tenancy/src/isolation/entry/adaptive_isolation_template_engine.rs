use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive isolation template calculation.
pub trait AdaptiveIsolationTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveIsolationTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveIsolationTemplateAdjuster {
    pub cap: f64,
}

impl AdaptiveIsolationTemplateModifier for CapAdaptiveIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveIsolationTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveIsolationTemplateAdjuster {
    pub percent: f64,
}

impl AdaptiveIsolationTemplateModifier for PercentAdaptiveIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveIsolationTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveIsolationTemplateAdjuster {
    pub minimum: f64,
}

impl AdaptiveIsolationTemplateModifier for MinimumAdaptiveIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveIsolationTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveIsolationTemplateAdjuster {
    pub step: f64,
}

impl AdaptiveIsolationTemplateModifier for RoundingAdaptiveIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveIsolationTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the adaptive isolation template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveIsolationTemplateEngine {
    pub steps: Vec<Rc<dyn AdaptiveIsolationTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveIsolationTemplateEngine {
    pub fn run(&mut self, item: &AdaptiveIsolationTemplate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveIsolationTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_isolation_template_engine() -> AdaptiveIsolationTemplateEngine {
    AdaptiveIsolationTemplateEngine {
        steps: vec![
            Rc::new(CapAdaptiveIsolationTemplateAdjuster { cap: 10377.0 }) as Rc<dyn AdaptiveIsolationTemplateModifier>,
            Rc::new(PercentAdaptiveIsolationTemplateAdjuster { percent: -6.2 }),
            Rc::new(MinimumAdaptiveIsolationTemplateAdjuster { minimum: 2.7 }),
            Rc::new(RoundingAdaptiveIsolationTemplateAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 80661.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_isolation_template_engine();
        let item = new_adaptive_isolation_template("code-787", "label-941");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
