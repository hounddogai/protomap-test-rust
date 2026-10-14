use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the isolation template calculation.
pub trait IsolationTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIsolationTemplateAdjuster {
    pub percent: f64,
}

impl IsolationTemplateModifier for PercentIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IsolationTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationTemplateModifier for TieredIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationTemplateAdjuster {
    pub minimum: f64,
}

impl IsolationTemplateModifier for MinimumIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationTemplateAdjuster {
    pub cap: f64,
}

impl IsolationTemplateModifier for CapIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationTemplateModifier for FlaggedIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleIsolationTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationTemplateModifier for TitleIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &IsolationTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the isolation template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationTemplateEngine {
    pub steps: Vec<Rc<dyn IsolationTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationTemplateEngine {
    pub fn evaluate(&mut self, item: &IsolationTemplate) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IsolationTemplateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IsolationTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_isolation_template_engine(flags: &flags::Registry) -> IsolationTemplateEngine {
    IsolationTemplateEngine {
        steps: vec![
            Rc::new(PercentIsolationTemplateAdjuster { percent: -16.4 }) as Rc<dyn IsolationTemplateModifier>,
            Rc::new(TieredIsolationTemplateAdjuster {
                bounds: vec![211.0, 4397.0, 8267.0],
                rates: vec![1.03, 0.92, 1.1],
            }),
            Rc::new(MinimumIsolationTemplateAdjuster { minimum: 20.8 }),
            Rc::new(CapIsolationTemplateAdjuster { cap: 40550.0 }),
            Rc::new(FlaggedIsolationTemplateAdjuster {
                flags: flags.clone(),
                flag: "isolation-template-default".to_string(),
                percent: 4.5,
            }),
            Rc::new(TitleIsolationTemplateAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.35)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 58660.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_template_engine(&flags::new_registry());
        let item = new_isolation_template("code-892", "title-843");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
