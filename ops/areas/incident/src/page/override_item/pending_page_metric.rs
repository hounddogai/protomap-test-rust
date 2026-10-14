use super::*;

use acme_kit::text;

/// Where a pending page metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPageMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingPageMetricStatus {
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

impl std::fmt::Display for PendingPageMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending page metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageMetric {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub locale: String,
    pub audience: String,
    pub quantity: i64,
    pub status: PendingPageMetricStatus,
}

impl PendingPageMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPageMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPageMetricStatus) -> bool {
        match self.status {
            PendingPageMetricStatus::Planned => {
                return next == PendingPageMetricStatus::InProgress;
            }
            PendingPageMetricStatus::Scheduled => {
                match next {
                    PendingPageMetricStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPageMetricStatus::InProgress => {
                match next {
                    PendingPageMetricStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPageMetricStatus::Completed => {
                match next {
                    PendingPageMetricStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PendingPageMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending page metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending page metric", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> PendingPageMetric {
        PendingPageMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            locale: self.locale.clone(),
            audience: self.audience.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a pending page metric in status planned.
pub fn new_pending_page_metric(code: &str, label: &str) -> PendingPageMetric {
    PendingPageMetric {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        locale: "".to_string(),
        audience: "".to_string(),
        quantity: 0,
        status: PendingPageMetricStatus::Planned,
    }
}
