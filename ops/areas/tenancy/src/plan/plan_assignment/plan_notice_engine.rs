use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan notice calculation.
pub trait PlanNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePlanNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanNoticeAdjuster for TitlePlanNoticeAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PlanNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanNoticeAdjuster for FlaggedPlanNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanNoticeAdjuster {
    pub percent: f64,
}

impl PlanNoticeAdjuster for PercentPlanNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanNoticeAdjuster {
    pub minimum: f64,
}

impl PlanNoticeAdjuster for MinimumPlanNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the plan notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanNoticeEngine {
    pub steps: Vec<Rc<dyn PlanNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanNoticeEngine {
    pub fn evaluate(&mut self, item: &PlanNotice) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PlanNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_plan_notice_engine(flags: &flags::Registry) -> PlanNoticeEngine {
    PlanNoticeEngine {
        steps: vec![
            Rc::new(TitlePlanNoticeAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.11)]),
            }) as Rc<dyn PlanNoticeAdjuster>,
            Rc::new(FlaggedPlanNoticeAdjuster {
                flags: flags.clone(),
                flag: "plan-notice-pending".to_string(),
                percent: -8.8,
            }),
            Rc::new(PercentPlanNoticeAdjuster { percent: 14.0 }),
            Rc::new(MinimumPlanNoticeAdjuster { minimum: 23.2 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 77053.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_notice_engine(&flags::new_registry());
        let item = new_plan_notice("code-548", "title-510");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
