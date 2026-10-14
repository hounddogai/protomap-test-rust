use super::*;

use acme_kit::text;

/// Where a batched allowlist entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAllowlistEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedAllowlistEntryStatus {
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

impl std::fmt::Display for BatchedAllowlistEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched allowlist entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistEntry {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub description: String,
    pub channel: String,
    pub status: BatchedAllowlistEntryStatus,
}

impl BatchedAllowlistEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAllowlistEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAllowlistEntryStatus) -> bool {
        match self.status {
            BatchedAllowlistEntryStatus::Planned => {
                return next == BatchedAllowlistEntryStatus::InProgress;
            }
            BatchedAllowlistEntryStatus::Scheduled => {
                match next {
                    BatchedAllowlistEntryStatus::InProgress | BatchedAllowlistEntryStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAllowlistEntryStatus::InProgress => {
                return next == BatchedAllowlistEntryStatus::Completed;
            }
            BatchedAllowlistEntryStatus::Completed => {
                match next {
                    BatchedAllowlistEntryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BatchedAllowlistEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched allowlist entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> BatchedAllowlistEntry {
        BatchedAllowlistEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            description: self.description.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a batched allowlist entry in status planned.
pub fn new_batched_allowlist_entry(code: &str, label: &str) -> BatchedAllowlistEntry {
    BatchedAllowlistEntry {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        description: "".to_string(),
        channel: "".to_string(),
        status: BatchedAllowlistEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_allowlist_entry("code-279", "label-278");
        assert_eq!(item.status, BatchedAllowlistEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_batched_allowlist_entry("code-279", "label-278");
        if item.move_to(BatchedAllowlistEntryStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedAllowlistEntryStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_allowlist_entry("code-279", "label-278");
        item.status = BatchedAllowlistEntryStatus::Cancelled;
        assert!(
            item.move_to(BatchedAllowlistEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
