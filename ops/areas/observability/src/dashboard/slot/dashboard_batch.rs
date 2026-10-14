use super::*;

use acme_kit::text;

/// Where a dashboard batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DashboardBatchStatus {
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

impl std::fmt::Display for DashboardBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBatch {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub phone: String,
    pub label: String,
    pub active: bool,
    pub unit: String,
    pub ratio: f64,
    pub description: String,
    pub tags: Vec<String>,
    pub priority: i64,
    pub status: DashboardBatchStatus,
}

impl DashboardBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardBatchStatus) -> bool {
        match self.status {
            DashboardBatchStatus::New => {
                return next == DashboardBatchStatus::Failed;
            }
            DashboardBatchStatus::Queued => {
                return next == DashboardBatchStatus::Failed;
            }
            DashboardBatchStatus::Running => {
                match next {
                    DashboardBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardBatchStatus::Done => {
                return next == DashboardBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.priority as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardBatch {
        DashboardBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            phone: self.phone.clone(),
            label: self.label.clone(),
            active: self.active,
            unit: self.unit.clone(),
            ratio: self.ratio,
            description: self.description.clone(),
            tags: self.tags.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a dashboard batch in status new.
pub fn new_dashboard_batch(code: &str, name: &str) -> DashboardBatch {
    DashboardBatch {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        phone: "".to_string(),
        label: "".to_string(),
        active: false,
        unit: "".to_string(),
        ratio: 0.0,
        description: "".to_string(),
        tags: Vec::new(),
        priority: 0,
        status: DashboardBatchStatus::New,
    }
}
