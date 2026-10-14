use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding template calculation.
pub trait BindingTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingTemplateAdjuster {
    pub cap: f64,
}

impl BindingTemplateFactor for CapBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingTemplateFactor for TieredBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBindingTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingTemplateFactor for SegmentBindingTemplateAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &BindingTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingTemplateAdjuster {
    pub step: f64,
}

impl BindingTemplateFactor for RoundingBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the binding template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingTemplateEngine {
    pub steps: Vec<Rc<dyn BindingTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingTemplateEngine {
    pub fn evaluate(&mut self, item: &BindingTemplate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BindingTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_binding_template_engine() -> BindingTemplateEngine {
    BindingTemplateEngine {
        steps: vec![
            Rc::new(CapBindingTemplateAdjuster { cap: 9931.0 }) as Rc<dyn BindingTemplateFactor>,
            Rc::new(TieredBindingTemplateAdjuster {
                bounds: vec![2382.0, 6296.0, 8078.0, 8673.0],
                rates: vec![0.99, 0.81, 0.97, 0.85],
            }),
            Rc::new(SegmentBindingTemplateAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.18)]),
            }),
            Rc::new(RoundingBindingTemplateAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 58075.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_template_engine();
        let item = new_binding_template("code-18", "title-533");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
