use super::*;

use acme_kit::text;

/// Where a zone metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ZoneMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ZoneMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMetric {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub category: String,
    pub country: String,
    pub rate: f64,
    pub port: i64,
    pub grace_days: i64,
    pub status: ZoneMetricStatus,
}

impl ZoneMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneMetricStatus) -> bool {
        match self.status {
            ZoneMetricStatus::New => {
                return next == ZoneMetricStatus::Running || next == ZoneMetricStatus::Done;
            }
            ZoneMetricStatus::Queued => {
                match next {
                    ZoneMetricStatus::Running | ZoneMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneMetricStatus::Running => {
                match next {
                    ZoneMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneMetricStatus::Done => {
                return next == ZoneMetricStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.rate * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneMetric {
        ZoneMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            category: self.category.clone(),
            country: self.country.clone(),
            rate: self.rate,
            port: self.port,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a zone metric in status new.
pub fn new_zone_metric(code: &str, name: &str) -> ZoneMetric {
    ZoneMetric {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        category: "".to_string(),
        country: "".to_string(),
        rate: 0.0,
        port: 0,
        grace_days: 0,
        status: ZoneMetricStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_zone_metric("code-531", "name-646");
        assert_eq!(item.status, ZoneMetricStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_zone_metric("code-531", "name-646");
        if item.move_to(ZoneMetricStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneMetricStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_zone_metric("code-531", "name-646");
        item.status = ZoneMetricStatus::Failed;
        assert!(item.move_to(ZoneMetricStatus::New).is_err(), "expected move to fail");
    }
}
