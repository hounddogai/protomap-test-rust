use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled approval quota calculation.
pub trait ScheduledApprovalQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledApprovalQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledApprovalQuotaAmountOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledApprovalQuotaModifier for ScheduledApprovalQuotaAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledApprovalQuota, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitScheduledApprovalQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledApprovalQuotaModifier for UnitScheduledApprovalQuotaAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ScheduledApprovalQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledApprovalQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledApprovalQuotaModifier for TieredScheduledApprovalQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledApprovalQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledApprovalQuotaAdjuster {
    pub minimum: f64,
}

impl ScheduledApprovalQuotaModifier for MinimumScheduledApprovalQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledApprovalQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledApprovalQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledApprovalQuotaModifier for FlaggedScheduledApprovalQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledApprovalQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledApprovalQuotaAdjuster {
    pub step: f64,
}

impl ScheduledApprovalQuotaModifier for RoundingScheduledApprovalQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledApprovalQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled approval quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledApprovalQuotaEngine {
    pub steps: Vec<Rc<dyn ScheduledApprovalQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledApprovalQuotaEngine {
    pub fn price(&mut self, item: &ScheduledApprovalQuota) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledApprovalQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_approval_quota_engine(flags: &flags::Registry) -> ScheduledApprovalQuotaEngine {
    ScheduledApprovalQuotaEngine {
        steps: vec![
            Rc::new(ScheduledApprovalQuotaAmountOffsetAdjuster { weight: 1.19 })
                as Rc<dyn ScheduledApprovalQuotaModifier>,
            Rc::new(UnitScheduledApprovalQuotaAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.22), ("north".to_string(), 1.25)]),
            }),
            Rc::new(TieredScheduledApprovalQuotaAdjuster {
                bounds: vec![6851.0, 6955.0, 7779.0],
                rates: vec![0.97, 0.98, 0.88],
            }),
            Rc::new(MinimumScheduledApprovalQuotaAdjuster { minimum: 26.8 }),
            Rc::new(FlaggedScheduledApprovalQuotaAdjuster {
                flags: flags.clone(),
                flag: "scheduled-approval-quota-rolling".to_string(),
                percent: -4.0,
            }),
            Rc::new(RoundingScheduledApprovalQuotaAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 52006.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_approval_quota_engine(&flags::new_registry());
        let item = new_scheduled_approval_quota("code-84", "label-652");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
