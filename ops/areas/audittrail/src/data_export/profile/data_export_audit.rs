use super::*;

use acme_kit::text;

/// Where a data export audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DataExportAuditStatus {
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

impl std::fmt::Display for DataExportAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAudit {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub checksum: String,
    pub sequence: i64,
    pub issuer: String,
    pub status: DataExportAuditStatus,
}

impl DataExportAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportAuditStatus) -> bool {
        match self.status {
            DataExportAuditStatus::Idle => {
                match next {
                    DataExportAuditStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportAuditStatus::Warming => {
                match next {
                    DataExportAuditStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportAuditStatus::Ready => {
                return next == DataExportAuditStatus::Draining;
            }
            DataExportAuditStatus::Draining => {
                return next == DataExportAuditStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> DataExportAudit {
        DataExportAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            checksum: self.checksum.clone(),
            sequence: self.sequence,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a data export audit in status idle.
pub fn new_data_export_audit(code: &str, title: &str) -> DataExportAudit {
    DataExportAudit {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        checksum: "".to_string(),
        sequence: 0,
        issuer: "".to_string(),
        status: DataExportAuditStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_data_export_audit("code-520", "title-976");
        assert_eq!(item.status, DataExportAuditStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_data_export_audit("code-520", "title-976");
        if item.move_to(DataExportAuditStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportAuditStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_data_export_audit("code-520", "title-976");
        item.status = DataExportAuditStatus::Stopped;
        assert!(
            item.move_to(DataExportAuditStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
