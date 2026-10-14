use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive grant template calculation.
pub trait AdaptiveGrantTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveGrantTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveGrantTemplateDiscountOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveGrantTemplateModifier for AdaptiveGrantTemplateDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveGrantTemplate, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveGrantTemplateAdjuster {
    pub step: f64,
}

impl AdaptiveGrantTemplateModifier for RoundingAdaptiveGrantTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveGrantTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveGrantTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveGrantTemplateModifier for FlaggedAdaptiveGrantTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveGrantTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveGrantTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveGrantTemplateModifier for TieredAdaptiveGrantTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveGrantTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveGrantTemplateAdjuster {
    pub minimum: f64,
}

impl AdaptiveGrantTemplateModifier for MinimumAdaptiveGrantTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveGrantTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveGrantTemplateAdjuster {
    pub cap: f64,
}

impl AdaptiveGrantTemplateModifier for CapAdaptiveGrantTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveGrantTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the adaptive grant template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveGrantTemplateEngine {
    pub steps: Vec<Rc<dyn AdaptiveGrantTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveGrantTemplateEngine {
    pub fn evaluate(&mut self, item: &AdaptiveGrantTemplate) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveGrantTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_grant_template_engine(flags: &flags::Registry) -> AdaptiveGrantTemplateEngine {
    AdaptiveGrantTemplateEngine {
        steps: vec![
            Rc::new(AdaptiveGrantTemplateDiscountOffsetAdjuster { weight: 1.04 })
                as Rc<dyn AdaptiveGrantTemplateModifier>,
            Rc::new(RoundingAdaptiveGrantTemplateAdjuster { step: 0.5 }),
            Rc::new(FlaggedAdaptiveGrantTemplateAdjuster {
                flags: flags.clone(),
                flag: "adaptive-grant-template-primary".to_string(),
                percent: -1.8,
            }),
            Rc::new(TieredAdaptiveGrantTemplateAdjuster {
                bounds: vec![4566.0, 9452.0],
                rates: vec![0.81, 0.92],
            }),
            Rc::new(MinimumAdaptiveGrantTemplateAdjuster { minimum: 33.8 }),
            Rc::new(CapAdaptiveGrantTemplateAdjuster { cap: 21165.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 32913.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_grant_template_engine(&flags::new_registry());
        let item = new_adaptive_grant_template("code-16", "title-636");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
