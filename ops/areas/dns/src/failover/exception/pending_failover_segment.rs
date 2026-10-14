use super::*;

use acme_kit::text;

/// Where a pending failover segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFailoverSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingFailoverSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingFailoverSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending failover segment as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFailoverSegment {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub maximum: f64,
    pub category: String,
    pub quota_units: i64,
    pub contact_email: String,
    pub sequence: i64,
    pub window_days: i64,
    pub status: PendingFailoverSegmentStatus,
}

impl PendingFailoverSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFailoverSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFailoverSegmentStatus) -> bool {
        match self.status {
            PendingFailoverSegmentStatus::Created => {
                match next {
                    PendingFailoverSegmentStatus::Validated | PendingFailoverSegmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFailoverSegmentStatus::Validated => {
                return next == PendingFailoverSegmentStatus::Published
                    || next == PendingFailoverSegmentStatus::Retired;
            }
            PendingFailoverSegmentStatus::Published => {
                match next {
                    PendingFailoverSegmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingFailoverSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending failover segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending failover segment", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.maximum * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFailoverSegment {
        PendingFailoverSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            maximum: self.maximum,
            category: self.category.clone(),
            quota_units: self.quota_units,
            contact_email: self.contact_email.clone(),
            sequence: self.sequence,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a pending failover segment in status created.
pub fn new_pending_failover_segment(code: &str, name: &str) -> PendingFailoverSegment {
    PendingFailoverSegment {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        maximum: 0.0,
        category: "".to_string(),
        quota_units: 0,
        contact_email: "".to_string(),
        sequence: 0,
        window_days: 0,
        status: PendingFailoverSegmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_failover_segment("code-811", "name-760");
        assert_eq!(item.status, PendingFailoverSegmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_pending_failover_segment("code-811", "name-760");
        if item.move_to(PendingFailoverSegmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingFailoverSegmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_failover_segment("code-811", "name-760");
        item.status = PendingFailoverSegmentStatus::Retired;
        assert!(
            item.move_to(PendingFailoverSegmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
