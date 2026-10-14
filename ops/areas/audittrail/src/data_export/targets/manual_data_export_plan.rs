use super::*;

use acme_kit::text;

/// Where a manual data export plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDataExportPlanStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualDataExportPlanStatus {
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

impl std::fmt::Display for ManualDataExportPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual data export plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportPlan {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub host: String,
    pub checksum: String,
    pub retries: i64,
    pub external_ref: String,
    pub namespace: String,
    pub segment: String,
    pub channel: String,
    pub active: bool,
    pub status: ManualDataExportPlanStatus,
}

impl ManualDataExportPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDataExportPlanStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDataExportPlanStatus) -> bool {
        match self.status {
            ManualDataExportPlanStatus::Idle => {
                match next {
                    ManualDataExportPlanStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDataExportPlanStatus::Warming => {
                return next == ManualDataExportPlanStatus::Draining || next == ManualDataExportPlanStatus::Stopped;
            }
            ManualDataExportPlanStatus::Ready => {
                match next {
                    ManualDataExportPlanStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDataExportPlanStatus::Draining => {
                match next {
                    ManualDataExportPlanStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ManualDataExportPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual data export plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual data export plan", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> ManualDataExportPlan {
        ManualDataExportPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            host: self.host.clone(),
            checksum: self.checksum.clone(),
            retries: self.retries,
            external_ref: self.external_ref.clone(),
            namespace: self.namespace.clone(),
            segment: self.segment.clone(),
            channel: self.channel.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a manual data export plan in status idle.
pub fn new_manual_data_export_plan(code: &str, label: &str) -> ManualDataExportPlan {
    ManualDataExportPlan {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        host: "".to_string(),
        checksum: "".to_string(),
        retries: 0,
        external_ref: "".to_string(),
        namespace: "".to_string(),
        segment: "".to_string(),
        channel: "".to_string(),
        active: false,
        status: ManualDataExportPlanStatus::Idle,
    }
}
