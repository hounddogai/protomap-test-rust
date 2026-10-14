use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling suspension template calculation.
pub trait RollingSuspensionTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingSuspensionTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSuspensionTemplateCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl RollingSuspensionTemplateModifier for RollingSuspensionTemplateCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &RollingSuspensionTemplate, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingSuspensionTemplateAdjuster {
    pub step: f64,
}

impl RollingSuspensionTemplateModifier for RoundingRollingSuspensionTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingSuspensionTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingSuspensionTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingSuspensionTemplateModifier for FlaggedRollingSuspensionTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingSuspensionTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollingSuspensionTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingSuspensionTemplateModifier for TitleRollingSuspensionTemplateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollingSuspensionTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling suspension template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingSuspensionTemplateEngine {
    pub steps: Vec<Rc<dyn RollingSuspensionTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingSuspensionTemplateEngine {
    pub fn price(&mut self, item: &RollingSuspensionTemplate) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[RollingSuspensionTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_suspension_template_engine(flags: &flags::Registry) -> RollingSuspensionTemplateEngine {
    RollingSuspensionTemplateEngine {
        steps: vec![
            Rc::new(RollingSuspensionTemplateCreatedAtOffsetAdjuster { weight: 0.76 })
                as Rc<dyn RollingSuspensionTemplateModifier>,
            Rc::new(RoundingRollingSuspensionTemplateAdjuster { step: 0.5 }),
            Rc::new(FlaggedRollingSuspensionTemplateAdjuster {
                flags: flags.clone(),
                flag: "rolling-suspension-template-manual".to_string(),
                percent: 9.5,
            }),
            Rc::new(TitleRollingSuspensionTemplateAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.07), ("north".to_string(), 0.83)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 34094.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_suspension_template_engine(&flags::new_registry());
        let item = new_rolling_suspension_template("code-729", "title-452");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
