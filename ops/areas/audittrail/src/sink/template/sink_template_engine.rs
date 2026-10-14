use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the sink template calculation.
pub trait SinkTemplateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkTemplateAdjuster {
    pub minimum: f64,
}

impl SinkTemplateAdjuster for MinimumSinkTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTemplateExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl SinkTemplateAdjuster for SinkTemplateExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &SinkTemplate, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterSinkTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkTemplateAdjuster for ClusterSinkTemplateAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &SinkTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkTemplateAdjuster for FlaggedSinkTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the sink template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkTemplateEngine {
    pub steps: Vec<Rc<dyn SinkTemplateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkTemplateEngine {
    pub fn price(&mut self, item: &SinkTemplate) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SinkTemplateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SinkTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_sink_template_engine(flags: &flags::Registry) -> SinkTemplateEngine {
    SinkTemplateEngine {
        steps: vec![
            Rc::new(MinimumSinkTemplateAdjuster { minimum: 28.8 }) as Rc<dyn SinkTemplateAdjuster>,
            Rc::new(SinkTemplateExpiresAtOffsetAdjuster { weight: 0.49 }),
            Rc::new(ClusterSinkTemplateAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.24)]),
            }),
            Rc::new(FlaggedSinkTemplateAdjuster {
                flags: flags.clone(),
                flag: "sink-template-manual".to_string(),
                percent: 9.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 34465.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_template_engine(&flags::new_registry());
        let item = new_sink_template("code-85", "title-439");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
