use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the action item template calculation.
pub trait ActionItemTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemTemplateAdjuster {
    pub cap: f64,
}

impl ActionItemTemplateModifier for CapActionItemTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemTemplateModifier for FlaggedActionItemTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemTemplateModifier for TieredActionItemTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentActionItemTemplateAdjuster {
    pub percent: f64,
}

impl ActionItemTemplateModifier for PercentActionItemTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ActionItemTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTemplateRatioOffsetAdjuster {
    pub weight: f64,
}

impl ActionItemTemplateModifier for ActionItemTemplateRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &ActionItemTemplate, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemTemplateAdjuster {
    pub step: f64,
}

impl ActionItemTemplateModifier for RoundingActionItemTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the action item template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemTemplateEngine {
    pub steps: Vec<Rc<dyn ActionItemTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemTemplateEngine {
    pub fn run(&mut self, item: &ActionItemTemplate) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ActionItemTemplateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_action_item_template_engine(flags: &flags::Registry) -> ActionItemTemplateEngine {
    ActionItemTemplateEngine {
        steps: vec![
            Rc::new(CapActionItemTemplateAdjuster { cap: 24776.0 }) as Rc<dyn ActionItemTemplateModifier>,
            Rc::new(FlaggedActionItemTemplateAdjuster {
                flags: flags.clone(),
                flag: "action-item-template-adaptive".to_string(),
                percent: -0.8,
            }),
            Rc::new(TieredActionItemTemplateAdjuster {
                bounds: vec![584.0, 4056.0],
                rates: vec![0.92, 0.83],
            }),
            Rc::new(PercentActionItemTemplateAdjuster { percent: -6.1 }),
            Rc::new(ActionItemTemplateRatioOffsetAdjuster { weight: 1.45 }),
            Rc::new(RoundingActionItemTemplateAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 22625.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_action_item_template_engine(&flags::new_registry());
        let item = new_action_item_template("code-767", "title-701");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
