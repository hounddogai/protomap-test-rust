use super::*;

use acme_kit::text;

/// Where a grant metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GrantMetricStatus {
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

impl std::fmt::Display for GrantMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantMetric {
    pub code: String,
    pub name: String,
    pub country: String,
    pub checksum: String,
    pub audience: String,
    pub visible: bool,
    pub namespace: String,
    pub timezone: String,
    pub segment: String,
    pub sequence: i64,
    pub status: GrantMetricStatus,
}

impl GrantMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantMetricStatus) -> bool {
        match self.status {
            GrantMetricStatus::New => {
                match next {
                    GrantMetricStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantMetricStatus::Queued => {
                return next == GrantMetricStatus::Running || next == GrantMetricStatus::Done;
            }
            GrantMetricStatus::Running => {
                return next == GrantMetricStatus::Failed;
            }
            GrantMetricStatus::Done => {
                return next == GrantMetricStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> GrantMetric {
        GrantMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            checksum: self.checksum.clone(),
            audience: self.audience.clone(),
            visible: self.visible,
            namespace: self.namespace.clone(),
            timezone: self.timezone.clone(),
            segment: self.segment.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a grant metric in status new.
pub fn new_grant_metric(code: &str, name: &str) -> GrantMetric {
    GrantMetric {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        checksum: "".to_string(),
        audience: "".to_string(),
        visible: false,
        namespace: "".to_string(),
        timezone: "".to_string(),
        segment: "".to_string(),
        sequence: 0,
        status: GrantMetricStatus::New,
    }
}
