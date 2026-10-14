use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the page forecast calculation.
pub trait PageForecastModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageForecast, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPageForecastAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PageForecastModifier for FlaggedPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PageForecast, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPageForecastAdjuster {
    pub step: f64,
}

impl PageForecastModifier for RoundingPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PageForecast, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPageForecastAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PageForecastModifier for LabelPageForecastAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PageForecast, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPageForecastAdjuster {
    pub percent: f64,
}

impl PageForecastModifier for PercentPageForecastAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PageForecast, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the page forecast steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageForecastEngine {
    pub steps: Vec<Rc<dyn PageForecastModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageForecastEngine {
    pub fn run(&mut self, item: &PageForecast) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PageForecastModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PageForecast]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_page_forecast_engine(flags: &flags::Registry) -> PageForecastEngine {
    PageForecastEngine {
        steps: vec![
            Rc::new(FlaggedPageForecastAdjuster {
                flags: flags.clone(),
                flag: "page-forecast-batched".to_string(),
                percent: -7.8,
            }) as Rc<dyn PageForecastModifier>,
            Rc::new(RoundingPageForecastAdjuster { step: 1.0 }),
            Rc::new(LabelPageForecastAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.79), ("online".to_string(), 0.72)]),
            }),
            Rc::new(PercentPageForecastAdjuster { percent: 13.4 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 13769.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_page_forecast_engine(&flags::new_registry());
        let item = new_page_forecast("code-13", "label-966");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
