use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a flow log record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRecord {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub limit: i64,
    pub ratio: f64,
    pub budget: f64,
    pub channel: String,
    pub name: String,
    pub key_id: String,
    pub created_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: FlowLogRecordStatus,
}

impl FlowLogRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogRecordStatus) -> bool {
        match self.status {
            FlowLogRecordStatus::Proposed => {
                return next == FlowLogRecordStatus::Withdrawn;
            }
            FlowLogRecordStatus::Approved => {
                return next == FlowLogRecordStatus::Rejected || next == FlowLogRecordStatus::Withdrawn;
            }
            FlowLogRecordStatus::Rejected => {
                return next == FlowLogRecordStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.budget + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogRecord {
        FlowLogRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            limit: self.limit,
            ratio: self.ratio,
            budget: self.budget,
            channel: self.channel.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            created_at: self.created_at,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a flow log record in status proposed.
pub fn new_flow_log_record(code: &str, title: &str) -> FlowLogRecord {
    FlowLogRecord {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        limit: 0,
        ratio: 0.0,
        budget: 0.0,
        channel: "".to_string(),
        name: "".to_string(),
        key_id: "".to_string(),
        created_at: 0,
        attributes: BTreeMap::new(),
        status: FlowLogRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_flow_log_record("code-723", "title-95");
        assert_eq!(item.status, FlowLogRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_flow_log_record("code-723", "title-95");
        if item.move_to(FlowLogRecordStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogRecordStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_flow_log_record("code-723", "title-95");
        item.status = FlowLogRecordStatus::Withdrawn;
        assert!(
            item.move_to(FlowLogRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
