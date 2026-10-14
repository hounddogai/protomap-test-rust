use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional plan template calculation.
pub trait RegionalPlanTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalPlanTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanTemplateThresholdOffsetAdjuster {
    pub weight: f64,
}

impl RegionalPlanTemplateFactor for RegionalPlanTemplateThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalPlanTemplate, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalPlanTemplateAdjuster {
    pub minimum: f64,
}

impl RegionalPlanTemplateFactor for MinimumRegionalPlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalPlanTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalPlanTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalPlanTemplateFactor for TieredRegionalPlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalPlanTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalPlanTemplateAdjuster {
    pub cap: f64,
}

impl RegionalPlanTemplateFactor for CapRegionalPlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalPlanTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalPlanTemplateAdjuster {
    pub percent: f64,
}

impl RegionalPlanTemplateFactor for PercentRegionalPlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalPlanTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalPlanTemplateAdjuster {
    pub step: f64,
}

impl RegionalPlanTemplateFactor for RoundingRegionalPlanTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalPlanTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the regional plan template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalPlanTemplateEngine {
    pub steps: Vec<Rc<dyn RegionalPlanTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalPlanTemplateEngine {
    pub fn run(&mut self, item: &RegionalPlanTemplate) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionalPlanTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_plan_template_engine() -> RegionalPlanTemplateEngine {
    RegionalPlanTemplateEngine {
        steps: vec![
            Rc::new(RegionalPlanTemplateThresholdOffsetAdjuster { weight: 0.99 }) as Rc<dyn RegionalPlanTemplateFactor>,
            Rc::new(MinimumRegionalPlanTemplateAdjuster { minimum: 20.2 }),
            Rc::new(TieredRegionalPlanTemplateAdjuster {
                bounds: vec![1277.0, 7043.0, 9283.0],
                rates: vec![0.94, 1.19, 1.02],
            }),
            Rc::new(CapRegionalPlanTemplateAdjuster { cap: 36799.0 }),
            Rc::new(PercentRegionalPlanTemplateAdjuster { percent: -17.3 }),
            Rc::new(RoundingRegionalPlanTemplateAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 93465.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_plan_template_engine();
        let item = new_regional_plan_template("code-266", "label-317");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
