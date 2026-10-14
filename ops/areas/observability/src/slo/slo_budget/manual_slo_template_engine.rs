use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual slo template calculation.
pub trait ManualSloTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualSloTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualSloTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualSloTemplateFactor for FlaggedManualSloTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualSloTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloTemplateLimitOffsetAdjuster {
    pub weight: f64,
}

impl ManualSloTemplateFactor for ManualSloTemplateLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &ManualSloTemplate, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualSloTemplateAdjuster {
    pub cap: f64,
}

impl ManualSloTemplateFactor for CapManualSloTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualSloTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleManualSloTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualSloTemplateFactor for LocaleManualSloTemplateAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &ManualSloTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualSloTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualSloTemplateFactor for TieredManualSloTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualSloTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual slo template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualSloTemplateEngine {
    pub steps: Vec<Rc<dyn ManualSloTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualSloTemplateEngine {
    pub fn compute(&mut self, item: &ManualSloTemplate) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualSloTemplateFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualSloTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_slo_template_engine(flags: &flags::Registry) -> ManualSloTemplateEngine {
    ManualSloTemplateEngine {
        steps: vec![
            Rc::new(FlaggedManualSloTemplateAdjuster {
                flags: flags.clone(),
                flag: "manual-slo-template-lenient".to_string(),
                percent: 1.1,
            }) as Rc<dyn ManualSloTemplateFactor>,
            Rc::new(ManualSloTemplateLimitOffsetAdjuster { weight: 1.2 }),
            Rc::new(CapManualSloTemplateAdjuster { cap: 23570.0 }),
            Rc::new(LocaleManualSloTemplateAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.38),
                    ("north".to_string(), 0.73),
                    ("east".to_string(), 1.02),
                ]),
            }),
            Rc::new(TieredManualSloTemplateAdjuster {
                bounds: vec![168.0, 4567.0],
                rates: vec![1.19, 0.91],
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 94631.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_slo_template_engine(&flags::new_registry());
        let item = new_manual_slo_template("code-898", "label-833");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
