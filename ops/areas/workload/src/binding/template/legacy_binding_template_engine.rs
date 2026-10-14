use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy binding template calculation.
pub trait LegacyBindingTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyBindingTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyBindingTemplateAdjuster {
    pub minimum: f64,
}

impl LegacyBindingTemplateModifier for MinimumLegacyBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyBindingTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyBindingTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyBindingTemplateModifier for TieredLegacyBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyBindingTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingTemplateQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl LegacyBindingTemplateModifier for LegacyBindingTemplateQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyBindingTemplate, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyBindingTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyBindingTemplateModifier for FlaggedLegacyBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyBindingTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyBindingTemplateAdjuster {
    pub percent: f64,
}

impl LegacyBindingTemplateModifier for PercentLegacyBindingTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyBindingTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLegacyBindingTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyBindingTemplateModifier for TitleLegacyBindingTemplateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LegacyBindingTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy binding template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyBindingTemplateEngine {
    pub steps: Vec<Rc<dyn LegacyBindingTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyBindingTemplateEngine {
    pub fn run(&mut self, item: &LegacyBindingTemplate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LegacyBindingTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_binding_template_engine(flags: &flags::Registry) -> LegacyBindingTemplateEngine {
    LegacyBindingTemplateEngine {
        steps: vec![
            Rc::new(MinimumLegacyBindingTemplateAdjuster { minimum: 34.5 }) as Rc<dyn LegacyBindingTemplateModifier>,
            Rc::new(TieredLegacyBindingTemplateAdjuster {
                bounds: vec![460.0, 3909.0, 6690.0, 8517.0],
                rates: vec![1.15, 1.19, 0.96, 0.9],
            }),
            Rc::new(LegacyBindingTemplateQuotaUnitsOffsetAdjuster { weight: 0.43 }),
            Rc::new(FlaggedLegacyBindingTemplateAdjuster {
                flags: flags.clone(),
                flag: "legacy-binding-template-manual".to_string(),
                percent: -3.1,
            }),
            Rc::new(PercentLegacyBindingTemplateAdjuster { percent: 17.4 }),
            Rc::new(TitleLegacyBindingTemplateAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.39)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 47398.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_binding_template_engine(&flags::new_registry());
        let item = new_legacy_binding_template("code-314", "title-490");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
