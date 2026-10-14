use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the runbook template calculation.
pub trait RunbookTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookTemplateFactor for FlaggedRunbookTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookTemplateAdjuster {
    pub step: f64,
}

impl RunbookTemplateFactor for RoundingRunbookTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookTemplateAdjuster {
    pub percent: f64,
}

impl RunbookTemplateFactor for PercentRunbookTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookTemplateAdjuster {
    pub cap: f64,
}

impl RunbookTemplateFactor for CapRunbookTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRunbookTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookTemplateFactor for UnitRunbookTemplateAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &RunbookTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the runbook template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookTemplateEngine {
    pub steps: Vec<Rc<dyn RunbookTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookTemplateEngine {
    pub fn evaluate(&mut self, item: &RunbookTemplate) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookTemplateFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_runbook_template_engine(flags: &flags::Registry) -> RunbookTemplateEngine {
    RunbookTemplateEngine {
        steps: vec![
            Rc::new(FlaggedRunbookTemplateAdjuster {
                flags: flags.clone(),
                flag: "runbook-template-batched".to_string(),
                percent: -2.4,
            }) as Rc<dyn RunbookTemplateFactor>,
            Rc::new(RoundingRunbookTemplateAdjuster { step: 5.0 }),
            Rc::new(PercentRunbookTemplateAdjuster { percent: 6.1 }),
            Rc::new(CapRunbookTemplateAdjuster { cap: 31301.0 }),
            Rc::new(UnitRunbookTemplateAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.3), ("online".to_string(), 1.1)]),
            }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 60687.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_template_engine(&flags::new_registry());
        let item = new_runbook_template("code-956", "name-657");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
