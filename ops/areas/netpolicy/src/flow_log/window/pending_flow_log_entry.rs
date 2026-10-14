use super::*;

use acme_kit::text;

/// Where a pending flow log entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFlowLogEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingFlowLogEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingFlowLogEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending flow log entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEntry {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub floor_value: f64,
    pub sequence: i64,
    pub approved: bool,
    pub revision: i64,
    pub rate: f64,
    pub region: String,
    pub owner_id: String,
    pub checksum: String,
    pub status: PendingFlowLogEntryStatus,
}

impl PendingFlowLogEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFlowLogEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFlowLogEntryStatus) -> bool {
        match self.status {
            PendingFlowLogEntryStatus::Created => {
                return next == PendingFlowLogEntryStatus::Validated || next == PendingFlowLogEntryStatus::Published;
            }
            PendingFlowLogEntryStatus::Validated => {
                return next == PendingFlowLogEntryStatus::Published || next == PendingFlowLogEntryStatus::Retired;
            }
            PendingFlowLogEntryStatus::Published => {
                return next == PendingFlowLogEntryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingFlowLogEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending flow log entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending flow log entry", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.revision as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFlowLogEntry {
        PendingFlowLogEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            floor_value: self.floor_value,
            sequence: self.sequence,
            approved: self.approved,
            revision: self.revision,
            rate: self.rate,
            region: self.region.clone(),
            owner_id: self.owner_id.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a pending flow log entry in status created.
pub fn new_pending_flow_log_entry(code: &str, label: &str) -> PendingFlowLogEntry {
    PendingFlowLogEntry {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        floor_value: 0.0,
        sequence: 0,
        approved: false,
        revision: 0,
        rate: 0.0,
        region: "".to_string(),
        owner_id: "".to_string(),
        checksum: "".to_string(),
        status: PendingFlowLogEntryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_flow_log_entry("code-377", "label-518");
        assert_eq!(item.status, PendingFlowLogEntryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_pending_flow_log_entry("code-377", "label-518");
        if item.move_to(PendingFlowLogEntryStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingFlowLogEntryStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_flow_log_entry("code-377", "label-518");
        item.status = PendingFlowLogEntryStatus::Retired;
        assert!(
            item.move_to(PendingFlowLogEntryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
