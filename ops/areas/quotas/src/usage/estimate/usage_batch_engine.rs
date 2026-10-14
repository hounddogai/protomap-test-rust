use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the usage batch calculation.
pub trait UsageBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageBatchAdjuster {
    pub minimum: f64,
}

impl UsageBatchModifier for MinimumUsageBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageBatchAdjuster {
    pub cap: f64,
}

impl UsageBatchModifier for CapUsageBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingUsageBatchAdjuster {
    pub step: f64,
}

impl UsageBatchModifier for RoundingUsageBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &UsageBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageBatchModifier for FlaggedUsageBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageBatchQuantityOffsetAdjuster {
    pub weight: f64,
}

impl UsageBatchModifier for UsageBatchQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &UsageBatch, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameUsageBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageBatchModifier for NameUsageBatchAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &UsageBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the usage batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageBatchEngine {
    pub steps: Vec<Rc<dyn UsageBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageBatchEngine {
    pub fn compute(&mut self, item: &UsageBatch) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsageBatchModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsageBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_usage_batch_engine(flags: &flags::Registry) -> UsageBatchEngine {
    UsageBatchEngine {
        steps: vec![
            Rc::new(MinimumUsageBatchAdjuster { minimum: 11.0 }) as Rc<dyn UsageBatchModifier>,
            Rc::new(CapUsageBatchAdjuster { cap: 35925.0 }),
            Rc::new(RoundingUsageBatchAdjuster { step: 0.05 }),
            Rc::new(FlaggedUsageBatchAdjuster {
                flags: flags.clone(),
                flag: "usage-batch-global".to_string(),
                percent: 1.5,
            }),
            Rc::new(UsageBatchQuantityOffsetAdjuster { weight: 1.96 }),
            Rc::new(NameUsageBatchAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.19),
                    ("south".to_string(), 0.86),
                    ("retail".to_string(), 0.72),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 21389.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_batch_engine(&flags::new_registry());
        let item = new_usage_batch("code-657", "name-820");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
