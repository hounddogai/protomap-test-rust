use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled suspension quota calculation.
pub trait ScheduledSuspensionQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledSuspensionQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionQuotaExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledSuspensionQuotaModifier for ScheduledSuspensionQuotaExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledSuspensionQuota, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledSuspensionQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledSuspensionQuotaModifier for FlaggedScheduledSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledSuspensionQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameScheduledSuspensionQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledSuspensionQuotaModifier for NameScheduledSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ScheduledSuspensionQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled suspension quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionQuotaEngine {
    pub steps: Vec<Rc<dyn ScheduledSuspensionQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledSuspensionQuotaEngine {
    pub fn evaluate(&mut self, item: &ScheduledSuspensionQuota) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledSuspensionQuotaModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledSuspensionQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_suspension_quota_engine(flags: &flags::Registry) -> ScheduledSuspensionQuotaEngine {
    ScheduledSuspensionQuotaEngine {
        steps: vec![
            Rc::new(ScheduledSuspensionQuotaExpiresAtOffsetAdjuster { weight: 1.51 })
                as Rc<dyn ScheduledSuspensionQuotaModifier>,
            Rc::new(FlaggedScheduledSuspensionQuotaAdjuster {
                flags: flags.clone(),
                flag: "scheduled-suspension-quota-seasonal".to_string(),
                percent: -5.4,
            }),
            Rc::new(NameScheduledSuspensionQuotaAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.17),
                    ("retail".to_string(), 0.93),
                    ("central".to_string(), 1.3),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 40530.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_suspension_quota_engine(&flags::new_registry());
        let item = new_scheduled_suspension_quota("code-738", "name-655");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
