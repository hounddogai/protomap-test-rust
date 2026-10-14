use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped dashboard template calculation.
pub trait CappedDashboardTemplateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedDashboardTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedDashboardTemplateAdjuster {
    pub step: f64,
}

impl CappedDashboardTemplateFactor for RoundingCappedDashboardTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedDashboardTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleCappedDashboardTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedDashboardTemplateFactor for TitleCappedDashboardTemplateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &CappedDashboardTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardTemplateWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl CappedDashboardTemplateFactor for CappedDashboardTemplateWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &CappedDashboardTemplate, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedDashboardTemplateAdjuster {
    pub percent: f64,
}

impl CappedDashboardTemplateFactor for PercentCappedDashboardTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedDashboardTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped dashboard template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedDashboardTemplateEngine {
    pub steps: Vec<Rc<dyn CappedDashboardTemplateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedDashboardTemplateEngine {
    pub fn run(&mut self, item: &CappedDashboardTemplate) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[CappedDashboardTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_dashboard_template_engine() -> CappedDashboardTemplateEngine {
    CappedDashboardTemplateEngine {
        steps: vec![
            Rc::new(RoundingCappedDashboardTemplateAdjuster { step: 0.1 }) as Rc<dyn CappedDashboardTemplateFactor>,
            Rc::new(TitleCappedDashboardTemplateAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.2),
                    ("central".to_string(), 0.8),
                    ("south".to_string(), 1.38),
                ]),
            }),
            Rc::new(CappedDashboardTemplateWindowDaysOffsetAdjuster { weight: 1.18 }),
            Rc::new(PercentCappedDashboardTemplateAdjuster { percent: 12.9 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 2772.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_dashboard_template_engine();
        let item = new_capped_dashboard_template("code-535", "title-150");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
