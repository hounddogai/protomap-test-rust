use super::*;

use acme_kit::text;

/// Where a default slo metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultSloMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultSloMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DefaultSloMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default slo metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloMetric {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub ceiling: f64,
    pub minimum: f64,
    pub archived: bool,
    pub weight: f64,
    pub status: DefaultSloMetricStatus,
}

impl DefaultSloMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultSloMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultSloMetricStatus) -> bool {
        match self.status {
            DefaultSloMetricStatus::Planned => {
                match next {
                    DefaultSloMetricStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSloMetricStatus::Scheduled => {
                match next {
                    DefaultSloMetricStatus::InProgress | DefaultSloMetricStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSloMetricStatus::InProgress => {
                return next == DefaultSloMetricStatus::Completed || next == DefaultSloMetricStatus::Cancelled;
            }
            DefaultSloMetricStatus::Completed => {
                match next {
                    DefaultSloMetricStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultSloMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default slo metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        (self.ceiling * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultSloMetric {
        DefaultSloMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            ceiling: self.ceiling,
            minimum: self.minimum,
            archived: self.archived,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default slo metric in status planned.
pub fn new_default_slo_metric(code: &str, title: &str) -> DefaultSloMetric {
    DefaultSloMetric {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        ceiling: 0.0,
        minimum: 0.0,
        archived: false,
        weight: 0.0,
        status: DefaultSloMetricStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_slo_metric("code-392", "title-870");
        assert_eq!(item.status, DefaultSloMetricStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_default_slo_metric("code-392", "title-870");
        if item.move_to(DefaultSloMetricStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultSloMetricStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_slo_metric("code-392", "title-870");
        item.status = DefaultSloMetricStatus::Cancelled;
        assert!(
            item.move_to(DefaultSloMetricStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
