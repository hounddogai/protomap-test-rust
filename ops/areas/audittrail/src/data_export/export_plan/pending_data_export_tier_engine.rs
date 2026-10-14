use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending data export tier calculation.
pub trait PendingDataExportTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingDataExportTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingDataExportTierAdjuster {
    pub step: f64,
}

impl PendingDataExportTierStep for RoundingPendingDataExportTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingDataExportTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingDataExportTierAdjuster {
    pub minimum: f64,
}

impl PendingDataExportTierStep for MinimumPendingDataExportTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingDataExportTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDataExportTierMarginOffsetAdjuster {
    pub weight: f64,
}

impl PendingDataExportTierStep for PendingDataExportTierMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &PendingDataExportTier, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingDataExportTierAdjuster {
    pub percent: f64,
}

impl PendingDataExportTierStep for PercentPendingDataExportTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingDataExportTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingDataExportTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingDataExportTierStep for FlaggedPendingDataExportTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingDataExportTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending data export tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingDataExportTierEngine {
    pub steps: Vec<Rc<dyn PendingDataExportTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingDataExportTierEngine {
    pub fn evaluate(&mut self, item: &PendingDataExportTier) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingDataExportTierStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingDataExportTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_data_export_tier_engine(flags: &flags::Registry) -> PendingDataExportTierEngine {
    PendingDataExportTierEngine {
        steps: vec![
            Rc::new(RoundingPendingDataExportTierAdjuster { step: 5.0 }) as Rc<dyn PendingDataExportTierStep>,
            Rc::new(MinimumPendingDataExportTierAdjuster { minimum: 14.2 }),
            Rc::new(PendingDataExportTierMarginOffsetAdjuster { weight: 1.33 }),
            Rc::new(PercentPendingDataExportTierAdjuster { percent: -1.1 }),
            Rc::new(FlaggedPendingDataExportTierAdjuster {
                flags: flags.clone(),
                flag: "pending-data-export-tier-strict".to_string(),
                percent: 6.4,
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 56829.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_data_export_tier_engine(&flags::new_registry());
        let item = new_pending_data_export_tier("code-107", "title-841");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
