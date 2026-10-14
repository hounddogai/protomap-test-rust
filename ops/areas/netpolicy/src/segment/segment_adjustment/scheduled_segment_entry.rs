use super::*;

use acme_kit::text;

/// Where a scheduled segment entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSegmentEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ScheduledSegmentEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ScheduledSegmentEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled segment entry as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentEntry {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub tenant_id: String,
    pub audience: String,
    pub key_id: String,
    pub amount: f64,
    pub status: ScheduledSegmentEntryStatus,
}

impl ScheduledSegmentEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSegmentEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSegmentEntryStatus) -> bool {
        match self.status {
            ScheduledSegmentEntryStatus::Requested => {
                match next {
                    ScheduledSegmentEntryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSegmentEntryStatus::Granted => {
                return next == ScheduledSegmentEntryStatus::Revoked || next == ScheduledSegmentEntryStatus::Expired;
            }
            ScheduledSegmentEntryStatus::Revoked => {
                match next {
                    ScheduledSegmentEntryStatus::Expired => {
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

    pub fn move_to(&mut self, next: ScheduledSegmentEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled segment entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled segment entry", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> ScheduledSegmentEntry {
        ScheduledSegmentEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            tenant_id: self.tenant_id.clone(),
            audience: self.audience.clone(),
            key_id: self.key_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled segment entry in status requested.
pub fn new_scheduled_segment_entry(code: &str, name: &str) -> ScheduledSegmentEntry {
    ScheduledSegmentEntry {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        tenant_id: "".to_string(),
        audience: "".to_string(),
        key_id: "".to_string(),
        amount: 0.0,
        status: ScheduledSegmentEntryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_scheduled_segment_entry("code-556", "name-656");
        assert_eq!(item.status, ScheduledSegmentEntryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_scheduled_segment_entry("code-556", "name-656");
        if item.move_to(ScheduledSegmentEntryStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSegmentEntryStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_scheduled_segment_entry("code-556", "name-656");
        item.status = ScheduledSegmentEntryStatus::Expired;
        assert!(
            item.move_to(ScheduledSegmentEntryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
