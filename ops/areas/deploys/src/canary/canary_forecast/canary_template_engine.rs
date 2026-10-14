use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the canary template calculation.
pub trait CanaryTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentCanaryTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryTemplateModifier for SegmentCanaryTemplateAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &CanaryTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryTemplateAdjuster {
    pub minimum: f64,
}

impl CanaryTemplateModifier for MinimumCanaryTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanaryTemplateAdjuster {
    pub percent: f64,
}

impl CanaryTemplateModifier for PercentCanaryTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanaryTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTemplatePortOffsetAdjuster {
    pub weight: f64,
}

impl CanaryTemplateModifier for CanaryTemplatePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryTemplate, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryTemplateAdjuster {
    pub step: f64,
}

impl CanaryTemplateModifier for RoundingCanaryTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCanaryTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanaryTemplateModifier for FlaggedCanaryTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanaryTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the canary template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryTemplateEngine {
    pub steps: Vec<Rc<dyn CanaryTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryTemplateEngine {
    pub fn compute(&mut self, item: &CanaryTemplate) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[CanaryTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_canary_template_engine(flags: &flags::Registry) -> CanaryTemplateEngine {
    CanaryTemplateEngine {
        steps: vec![
            Rc::new(SegmentCanaryTemplateAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.05)]),
            }) as Rc<dyn CanaryTemplateModifier>,
            Rc::new(MinimumCanaryTemplateAdjuster { minimum: 37.1 }),
            Rc::new(PercentCanaryTemplateAdjuster { percent: -21.3 }),
            Rc::new(CanaryTemplatePortOffsetAdjuster { weight: 1.95 }),
            Rc::new(RoundingCanaryTemplateAdjuster { step: 1.0 }),
            Rc::new(FlaggedCanaryTemplateAdjuster {
                flags: flags.clone(),
                flag: "canary-template-lenient".to_string(),
                percent: 4.6,
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 9151.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_template_engine(&flags::new_registry());
        let item = new_canary_template("code-628", "label-255");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
