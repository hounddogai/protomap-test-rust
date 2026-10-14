use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling page template calculation.
pub trait RollingPageTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPageTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPageTemplateAdjuster {
    pub step: f64,
}

impl RollingPageTemplateFactor for RoundingRollingPageTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPageTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRollingPageTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingPageTemplateFactor for LabelRollingPageTemplateAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RollingPageTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingPageTemplateAdjuster {
    pub cap: f64,
}

impl RollingPageTemplateFactor for CapRollingPageTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingPageTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPageTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPageTemplateFactor for TieredRollingPageTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPageTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingPageTemplateAdjuster {
    pub percent: f64,
}

impl RollingPageTemplateFactor for PercentRollingPageTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingPageTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageTemplateWeightOffsetAdjuster {
    pub weight: f64,
}

impl RollingPageTemplateFactor for RollingPageTemplateWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &RollingPageTemplate, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

/// Runs the rolling page template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPageTemplateEngine {
    pub steps: Vec<Rc<dyn RollingPageTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPageTemplateEngine {
    pub fn run(&mut self, item: &RollingPageTemplate) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingPageTemplateFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingPageTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_page_template_engine() -> RollingPageTemplateEngine {
    RollingPageTemplateEngine {
        steps: vec![
            Rc::new(RoundingRollingPageTemplateAdjuster { step: 0.5 }) as Rc<dyn RollingPageTemplateFactor>,
            Rc::new(LabelRollingPageTemplateAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.26)]),
            }),
            Rc::new(CapRollingPageTemplateAdjuster { cap: 13832.0 }),
            Rc::new(TieredRollingPageTemplateAdjuster {
                bounds: vec![822.0, 9350.0, 9424.0],
                rates: vec![1.01, 1.09, 1.13],
            }),
            Rc::new(PercentRollingPageTemplateAdjuster { percent: -6.9 }),
            Rc::new(RollingPageTemplateWeightOffsetAdjuster { weight: 1.11 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 41147.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_page_template_engine();
        let item = new_rolling_page_template("code-890", "label-168");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
