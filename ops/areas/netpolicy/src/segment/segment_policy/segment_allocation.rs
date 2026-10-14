use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a segment allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SegmentAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SegmentAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAllocation {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub ends_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub grace_days: i64,
    pub timezone: String,
    pub status: SegmentAllocationStatus,
}

impl SegmentAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentAllocationStatus) -> bool {
        match self.status {
            SegmentAllocationStatus::Requested => {
                match next {
                    SegmentAllocationStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentAllocationStatus::Granted => {
                match next {
                    SegmentAllocationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentAllocationStatus::Revoked => {
                match next {
                    SegmentAllocationStatus::Expired => {
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

    pub fn move_to(&mut self, next: SegmentAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn effective_load(&self) -> f64 {
        ((self.grace_days as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentAllocation {
        SegmentAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            ends_at: self.ends_at,
            attributes: self.attributes.clone(),
            grace_days: self.grace_days,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a segment allocation in status requested.
pub fn new_segment_allocation(code: &str, title: &str) -> SegmentAllocation {
    SegmentAllocation {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        ends_at: 0,
        attributes: BTreeMap::new(),
        grace_days: 0,
        timezone: "".to_string(),
        status: SegmentAllocationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_segment_allocation("code-650", "title-708");
        assert_eq!(item.status, SegmentAllocationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_segment_allocation("code-650", "title-708");
        if item.move_to(SegmentAllocationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentAllocationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_segment_allocation("code-650", "title-708");
        item.status = SegmentAllocationStatus::Expired;
        assert!(
            item.move_to(SegmentAllocationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
