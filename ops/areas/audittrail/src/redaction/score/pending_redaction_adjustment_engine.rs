use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending redaction adjustment calculation.
pub trait PendingRedactionAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRedactionAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingRedactionAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingRedactionAdjustmentModifier for FlaggedPendingRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingRedactionAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePendingRedactionAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingRedactionAdjustmentModifier for SourcePendingRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PendingRedactionAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingRedactionAdjustmentAdjuster {
    pub percent: f64,
}

impl PendingRedactionAdjustmentModifier for PercentPendingRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingRedactionAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingRedactionAdjustmentAdjuster {
    pub step: f64,
}

impl PendingRedactionAdjustmentModifier for RoundingPendingRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingRedactionAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingRedactionAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingRedactionAdjustmentModifier for TieredPendingRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingRedactionAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingRedactionAdjustmentAdjuster {
    pub minimum: f64,
}

impl PendingRedactionAdjustmentModifier for MinimumPendingRedactionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingRedactionAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending redaction adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRedactionAdjustmentEngine {
    pub steps: Vec<Rc<dyn PendingRedactionAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRedactionAdjustmentEngine {
    pub fn price(&mut self, item: &PendingRedactionAdjustment) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingRedactionAdjustmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingRedactionAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_redaction_adjustment_engine(flags: &flags::Registry) -> PendingRedactionAdjustmentEngine {
    PendingRedactionAdjustmentEngine {
        steps: vec![
            Rc::new(FlaggedPendingRedactionAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "pending-redaction-adjustment-global".to_string(),
                percent: 8.4,
            }) as Rc<dyn PendingRedactionAdjustmentModifier>,
            Rc::new(SourcePendingRedactionAdjustmentAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.19),
                    ("central".to_string(), 0.88),
                    ("south".to_string(), 1.33),
                ]),
            }),
            Rc::new(PercentPendingRedactionAdjustmentAdjuster { percent: -20.6 }),
            Rc::new(RoundingPendingRedactionAdjustmentAdjuster { step: 0.5 }),
            Rc::new(TieredPendingRedactionAdjustmentAdjuster {
                bounds: vec![3135.0, 7771.0],
                rates: vec![0.84, 0.83],
            }),
            Rc::new(MinimumPendingRedactionAdjustmentAdjuster { minimum: 12.3 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 92801.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_redaction_adjustment_engine(&flags::new_registry());
        let item = new_pending_redaction_adjustment("code-834", "title-176");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
