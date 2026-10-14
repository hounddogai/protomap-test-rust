use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient audit event group calculation.
pub trait LenientAuditEventGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientAuditEventGroup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientAuditEventGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientAuditEventGroupFactor for FlaggedLenientAuditEventGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientAuditEventGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientAuditEventGroupAdjuster {
    pub step: f64,
}

impl LenientAuditEventGroupFactor for RoundingLenientAuditEventGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientAuditEventGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientAuditEventGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientAuditEventGroupFactor for TieredLenientAuditEventGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientAuditEventGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAuditEventGroupGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl LenientAuditEventGroupFactor for LenientAuditEventGroupGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &LenientAuditEventGroup, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientAuditEventGroupAdjuster {
    pub percent: f64,
}

impl LenientAuditEventGroupFactor for PercentLenientAuditEventGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientAuditEventGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionLenientAuditEventGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientAuditEventGroupFactor for RegionLenientAuditEventGroupAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &LenientAuditEventGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient audit event group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientAuditEventGroupEngine {
    pub steps: Vec<Rc<dyn LenientAuditEventGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientAuditEventGroupEngine {
    pub fn run(&mut self, item: &LenientAuditEventGroup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientAuditEventGroupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientAuditEventGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_audit_event_group_engine(flags: &flags::Registry) -> LenientAuditEventGroupEngine {
    LenientAuditEventGroupEngine {
        steps: vec![
            Rc::new(FlaggedLenientAuditEventGroupAdjuster {
                flags: flags.clone(),
                flag: "lenient-audit-event-group-strict".to_string(),
                percent: 6.9,
            }) as Rc<dyn LenientAuditEventGroupFactor>,
            Rc::new(RoundingLenientAuditEventGroupAdjuster { step: 5.0 }),
            Rc::new(TieredLenientAuditEventGroupAdjuster {
                bounds: vec![6765.0, 6792.0, 9378.0, 9432.0],
                rates: vec![0.9, 1.04, 1.07, 0.98],
            }),
            Rc::new(LenientAuditEventGroupGraceDaysOffsetAdjuster { weight: 0.03 }),
            Rc::new(PercentLenientAuditEventGroupAdjuster { percent: 20.2 }),
            Rc::new(RegionLenientAuditEventGroupAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.33),
                    ("east".to_string(), 0.73),
                    ("west".to_string(), 0.93),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 68046.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_audit_event_group_engine(&flags::new_registry());
        let item = new_lenient_audit_event_group("code-915", "title-844");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
