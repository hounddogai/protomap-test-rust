use super::*;

use acme_kit::text;

/// Where a pending timeline record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTimelineRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingTimelineRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PendingTimelineRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending timeline record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineRecord {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub sequence: i64,
    pub discount: f64,
    pub fingerprint: String,
    pub title: String,
    pub approved: bool,
    pub tenant_id: String,
    pub status: PendingTimelineRecordStatus,
}

impl PendingTimelineRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTimelineRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTimelineRecordStatus) -> bool {
        match self.status {
            PendingTimelineRecordStatus::Draft => {
                return next == PendingTimelineRecordStatus::Pending || next == PendingTimelineRecordStatus::Active;
            }
            PendingTimelineRecordStatus::Pending => {
                return next == PendingTimelineRecordStatus::Active || next == PendingTimelineRecordStatus::Suspended;
            }
            PendingTimelineRecordStatus::Active => {
                match next {
                    PendingTimelineRecordStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTimelineRecordStatus::Suspended => {
                return next == PendingTimelineRecordStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingTimelineRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending timeline record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.discount + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingTimelineRecord {
        PendingTimelineRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            sequence: self.sequence,
            discount: self.discount,
            fingerprint: self.fingerprint.clone(),
            title: self.title.clone(),
            approved: self.approved,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending timeline record in status draft.
pub fn new_pending_timeline_record(code: &str, label: &str) -> PendingTimelineRecord {
    PendingTimelineRecord {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        sequence: 0,
        discount: 0.0,
        fingerprint: "".to_string(),
        title: "".to_string(),
        approved: false,
        tenant_id: "".to_string(),
        status: PendingTimelineRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_timeline_record("code-34", "label-563");
        assert_eq!(item.status, PendingTimelineRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_pending_timeline_record("code-34", "label-563");
        if item.move_to(PendingTimelineRecordStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingTimelineRecordStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_timeline_record("code-34", "label-563");
        item.status = PendingTimelineRecordStatus::Closed;
        assert!(
            item.move_to(PendingTimelineRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
