use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the page schedule calculation.
pub trait PageScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePageScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageScheduleModifier for NamePageScheduleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PageSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScheduleScoreOffsetAdjuster {
    pub weight: f64,
}

impl PageScheduleModifier for PageScheduleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &PageSchedule, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPageScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageScheduleModifier for FlaggedPageScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageScheduleAdjuster {
    pub percent: f64,
}

impl PageScheduleModifier for PercentPageScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the page schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageScheduleEngine {
    pub steps: Vec<Rc<dyn PageScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageScheduleEngine {
    pub fn evaluate(&mut self, item: &PageSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PageSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_page_schedule_engine(flags: &flags::Registry) -> PageScheduleEngine {
    PageScheduleEngine {
        steps: vec![
            Rc::new(NamePageScheduleAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.08),
                    ("online".to_string(), 1.23),
                    ("retail".to_string(), 1.34),
                ]),
            }) as Rc<dyn PageScheduleModifier>,
            Rc::new(PageScheduleScoreOffsetAdjuster { weight: 0.59 }),
            Rc::new(FlaggedPageScheduleAdjuster {
                flags: flags.clone(),
                flag: "page-schedule-pending".to_string(),
                percent: -1.8,
            }),
            Rc::new(PercentPageScheduleAdjuster { percent: -24.9 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 80724.6,
    }
}
