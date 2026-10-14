use super::*;

use acme_kit::text;

/// Where a flow log mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log mapping as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMapping {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub starts_at: i64,
    pub namespace: String,
    pub pinned: bool,
    pub status: FlowLogMappingStatus,
}

impl FlowLogMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogMappingStatus) -> bool {
        match self.status {
            FlowLogMappingStatus::Proposed => {
                return next == FlowLogMappingStatus::Rejected;
            }
            FlowLogMappingStatus::Approved => {
                return next == FlowLogMappingStatus::Withdrawn;
            }
            FlowLogMappingStatus::Rejected => {
                match next {
                    FlowLogMappingStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FlowLogMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> FlowLogMapping {
        FlowLogMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            starts_at: self.starts_at,
            namespace: self.namespace.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a flow log mapping in status proposed.
pub fn new_flow_log_mapping(code: &str, label: &str) -> FlowLogMapping {
    FlowLogMapping {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        starts_at: 0,
        namespace: "".to_string(),
        pinned: false,
        status: FlowLogMappingStatus::Proposed,
    }
}
