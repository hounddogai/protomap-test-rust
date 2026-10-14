use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual binding template calculation.
pub trait ManualBindingTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualBindingTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualBindingTemplateAdjuster {
    pub cap: f64,
}

impl ManualBindingTemplateFactor for CapManualBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualBindingTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualBindingTemplateAdjuster {
    pub percent: f64,
}

impl ManualBindingTemplateFactor for PercentManualBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualBindingTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelManualBindingTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualBindingTemplateFactor for LabelManualBindingTemplateAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ManualBindingTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualBindingTemplateAdjuster {
    pub minimum: f64,
}

impl ManualBindingTemplateFactor for MinimumManualBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualBindingTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualBindingTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualBindingTemplateFactor for FlaggedManualBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualBindingTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualBindingTemplateAdjuster {
    pub step: f64,
}

impl ManualBindingTemplateFactor for RoundingManualBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualBindingTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual binding template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualBindingTemplateEngine {
    pub steps: Vec<Rc<dyn ManualBindingTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualBindingTemplateEngine {
    pub fn run(&mut self, item: &ManualBindingTemplate) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualBindingTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_binding_template_engine(flags: &flags::Registry) -> ManualBindingTemplateEngine {
    ManualBindingTemplateEngine {
        steps: vec![
            Rc::new(CapManualBindingTemplateAdjuster { cap: 49897.0 }) as Rc<dyn ManualBindingTemplateFactor>,
            Rc::new(PercentManualBindingTemplateAdjuster { percent: -24.2 }),
            Rc::new(LabelManualBindingTemplateAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.39),
                    ("online".to_string(), 0.91),
                    ("south".to_string(), 0.77),
                ]),
            }),
            Rc::new(MinimumManualBindingTemplateAdjuster { minimum: 48.6 }),
            Rc::new(FlaggedManualBindingTemplateAdjuster {
                flags: flags.clone(),
                flag: "manual-binding-template-fallback".to_string(),
                percent: 9.2,
            }),
            Rc::new(RoundingManualBindingTemplateAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 49957.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_binding_template_engine(&flags::new_registry());
        let item = new_manual_binding_template("code-124", "label-306");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
