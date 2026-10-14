use super::*;

use acme_kit::text;

/// Where a rolling dns record item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingDnsRecordItemStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingDnsRecordItemStatus {
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

impl std::fmt::Display for RollingDnsRecordItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling dns record item. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDnsRecordItem {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub burst: i64,
    pub host: String,
    pub status: RollingDnsRecordItemStatus,
}

impl RollingDnsRecordItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingDnsRecordItemStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingDnsRecordItemStatus) -> bool {
        match self.status {
            RollingDnsRecordItemStatus::Planned => {
                match next {
                    RollingDnsRecordItemStatus::Completed | RollingDnsRecordItemStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDnsRecordItemStatus::Scheduled => {
                return next == RollingDnsRecordItemStatus::Cancelled;
            }
            RollingDnsRecordItemStatus::InProgress => {
                return next == RollingDnsRecordItemStatus::Completed || next == RollingDnsRecordItemStatus::Cancelled;
            }
            RollingDnsRecordItemStatus::Completed => {
                return next == RollingDnsRecordItemStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingDnsRecordItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling dns record item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> RollingDnsRecordItem {
        RollingDnsRecordItem {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            burst: self.burst,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling dns record item in status planned.
pub fn new_rolling_dns_record_item(code: &str, label: &str) -> RollingDnsRecordItem {
    RollingDnsRecordItem {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        burst: 0,
        host: "".to_string(),
        status: RollingDnsRecordItemStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_dns_record_item("code-27", "label-353");
        assert_eq!(item.status, RollingDnsRecordItemStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_rolling_dns_record_item("code-27", "label-353");
        if item.move_to(RollingDnsRecordItemStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingDnsRecordItemStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_dns_record_item("code-27", "label-353");
        item.status = RollingDnsRecordItemStatus::Cancelled;
        assert!(
            item.move_to(RollingDnsRecordItemStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
