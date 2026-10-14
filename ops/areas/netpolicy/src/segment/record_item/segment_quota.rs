use super::*;

use acme_kit::text;

/// Where a segment quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SegmentQuotaStatus {
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

impl std::fmt::Display for SegmentQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a segment quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentQuota {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub approved: bool,
    pub retries: i64,
    pub status: SegmentQuotaStatus,
}

impl SegmentQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentQuotaStatus) -> bool {
        match self.status {
            SegmentQuotaStatus::Draft => {
                return next == SegmentQuotaStatus::Active || next == SegmentQuotaStatus::Closed;
            }
            SegmentQuotaStatus::Pending => {
                match next {
                    SegmentQuotaStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentQuotaStatus::Active => {
                return next == SegmentQuotaStatus::Suspended;
            }
            SegmentQuotaStatus::Suspended => {
                return next == SegmentQuotaStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> SegmentQuota {
        SegmentQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            approved: self.approved,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a segment quota in status draft.
pub fn new_segment_quota(code: &str, name: &str) -> SegmentQuota {
    SegmentQuota {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        approved: false,
        retries: 0,
        status: SegmentQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_segment_quota("code-891", "name-541");
        assert_eq!(item.status, SegmentQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_segment_quota("code-891", "name-541");
        if item.move_to(SegmentQuotaStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentQuotaStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_segment_quota("code-891", "name-541");
        item.status = SegmentQuotaStatus::Closed;
        assert!(
            item.move_to(SegmentQuotaStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
