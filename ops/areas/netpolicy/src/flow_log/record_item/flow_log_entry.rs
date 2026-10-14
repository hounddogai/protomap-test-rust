use super::*;

use acme_kit::text;

/// Where a flow log entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a flow log entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEntry {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub reviewer: String,
    pub amount: f64,
    pub contact_email: String,
    pub budget: f64,
    pub version: i64,
    pub currency: String,
    pub source: String,
    pub status: FlowLogEntryStatus,
}

impl FlowLogEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogEntryStatus) -> bool {
        match self.status {
            FlowLogEntryStatus::Proposed => {
                match next {
                    FlowLogEntryStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogEntryStatus::Approved => {
                match next {
                    FlowLogEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogEntryStatus::Rejected => {
                match next {
                    FlowLogEntryStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FlowLogEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log entry", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ends_at as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogEntry {
        FlowLogEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            reviewer: self.reviewer.clone(),
            amount: self.amount,
            contact_email: self.contact_email.clone(),
            budget: self.budget,
            version: self.version,
            currency: self.currency.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a flow log entry in status proposed.
pub fn new_flow_log_entry(code: &str, title: &str) -> FlowLogEntry {
    FlowLogEntry {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        reviewer: "".to_string(),
        amount: 0.0,
        contact_email: "".to_string(),
        budget: 0.0,
        version: 0,
        currency: "".to_string(),
        source: "".to_string(),
        status: FlowLogEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_flow_log_entry("code-985", "title-604");
        assert_eq!(item.status, FlowLogEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_flow_log_entry("code-985", "title-604");
        if item.move_to(FlowLogEntryStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogEntryStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_flow_log_entry("code-985", "title-604");
        item.status = FlowLogEntryStatus::Withdrawn;
        assert!(
            item.move_to(FlowLogEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
