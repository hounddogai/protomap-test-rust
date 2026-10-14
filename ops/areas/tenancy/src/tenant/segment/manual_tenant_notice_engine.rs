use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual tenant notice calculation.
pub trait ManualTenantNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualTenantNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameManualTenantNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualTenantNoticeStep for NameManualTenantNoticeAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ManualTenantNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualTenantNoticeAdjuster {
    pub step: f64,
}

impl ManualTenantNoticeStep for RoundingManualTenantNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualTenantNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualTenantNoticeAdjuster {
    pub minimum: f64,
}

impl ManualTenantNoticeStep for MinimumManualTenantNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualTenantNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualTenantNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualTenantNoticeStep for FlaggedManualTenantNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualTenantNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualTenantNoticeAdjuster {
    pub cap: f64,
}

impl ManualTenantNoticeStep for CapManualTenantNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualTenantNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the manual tenant notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualTenantNoticeEngine {
    pub steps: Vec<Rc<dyn ManualTenantNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualTenantNoticeEngine {
    pub fn price(&mut self, item: &ManualTenantNotice) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualTenantNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualTenantNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_tenant_notice_engine(flags: &flags::Registry) -> ManualTenantNoticeEngine {
    ManualTenantNoticeEngine {
        steps: vec![
            Rc::new(NameManualTenantNoticeAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.9)]),
            }) as Rc<dyn ManualTenantNoticeStep>,
            Rc::new(RoundingManualTenantNoticeAdjuster { step: 1.0 }),
            Rc::new(MinimumManualTenantNoticeAdjuster { minimum: 22.4 }),
            Rc::new(FlaggedManualTenantNoticeAdjuster {
                flags: flags.clone(),
                flag: "manual-tenant-notice-scheduled".to_string(),
                percent: -8.5,
            }),
            Rc::new(CapManualTenantNoticeAdjuster { cap: 29994.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 5237.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_tenant_notice_engine(&flags::new_registry());
        let item = new_manual_tenant_notice("code-899", "name-170");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
