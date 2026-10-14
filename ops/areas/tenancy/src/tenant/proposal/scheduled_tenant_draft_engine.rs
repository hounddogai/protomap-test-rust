use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled tenant draft calculation.
pub trait ScheduledTenantDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledTenantDraft, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledTenantDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledTenantDraftModifier for FlaggedScheduledTenantDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledTenantDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantDraftCostOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledTenantDraftModifier for ScheduledTenantDraftCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledTenantDraft, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledTenantDraftAdjuster {
    pub minimum: f64,
}

impl ScheduledTenantDraftModifier for MinimumScheduledTenantDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledTenantDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledTenantDraftAdjuster {
    pub cap: f64,
}

impl ScheduledTenantDraftModifier for CapScheduledTenantDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledTenantDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled tenant draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledTenantDraftEngine {
    pub steps: Vec<Rc<dyn ScheduledTenantDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledTenantDraftEngine {
    pub fn compute(&mut self, item: &ScheduledTenantDraft) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledTenantDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_tenant_draft_engine(flags: &flags::Registry) -> ScheduledTenantDraftEngine {
    ScheduledTenantDraftEngine {
        steps: vec![
            Rc::new(FlaggedScheduledTenantDraftAdjuster {
                flags: flags.clone(),
                flag: "scheduled-tenant-draft-rolling".to_string(),
                percent: 1.3,
            }) as Rc<dyn ScheduledTenantDraftModifier>,
            Rc::new(ScheduledTenantDraftCostOffsetAdjuster { weight: 1.26 }),
            Rc::new(MinimumScheduledTenantDraftAdjuster { minimum: 30.4 }),
            Rc::new(CapScheduledTenantDraftAdjuster { cap: 33250.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 84034.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_tenant_draft_engine(&flags::new_registry());
        let item = new_scheduled_tenant_draft("code-805", "name-342");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
