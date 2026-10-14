use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled data export summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDataExportSummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledDataExportSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ScheduledDataExportSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled data export summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportSummary {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub summary: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub discount: f64,
    pub grace_days: i64,
    pub status: ScheduledDataExportSummaryStatus,
}

impl ScheduledDataExportSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDataExportSummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDataExportSummaryStatus) -> bool {
        match self.status {
            ScheduledDataExportSummaryStatus::Idle => {
                return next == ScheduledDataExportSummaryStatus::Warming
                    || next == ScheduledDataExportSummaryStatus::Stopped;
            }
            ScheduledDataExportSummaryStatus::Warming => {
                return next == ScheduledDataExportSummaryStatus::Ready
                    || next == ScheduledDataExportSummaryStatus::Stopped;
            }
            ScheduledDataExportSummaryStatus::Ready => {
                match next {
                    ScheduledDataExportSummaryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDataExportSummaryStatus::Draining => {
                return next == ScheduledDataExportSummaryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledDataExportSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled data export summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_load(&self) -> f64 {
        ((self.grace_days as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDataExportSummary {
        ScheduledDataExportSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            summary: self.summary.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            discount: self.discount,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled data export summary in status idle.
pub fn new_scheduled_data_export_summary(code: &str, label: &str) -> ScheduledDataExportSummary {
    ScheduledDataExportSummary {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        summary: "".to_string(),
        name: "".to_string(),
        attributes: BTreeMap::new(),
        discount: 0.0,
        grace_days: 0,
        status: ScheduledDataExportSummaryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_data_export_summary("code-356", "label-666");
        assert_eq!(item.status, ScheduledDataExportSummaryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_scheduled_data_export_summary("code-356", "label-666");
        if item.move_to(ScheduledDataExportSummaryStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledDataExportSummaryStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_data_export_summary("code-356", "label-666");
        item.status = ScheduledDataExportSummaryStatus::Stopped;
        assert!(
            item.move_to(ScheduledDataExportSummaryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
