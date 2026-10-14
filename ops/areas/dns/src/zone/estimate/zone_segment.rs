use super::*;

use acme_kit::text;

/// Where a zone segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ZoneSegmentStatus {
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

impl std::fmt::Display for ZoneSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone segment as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSegment {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub batch_size: i64,
    pub external_ref: String,
    pub status: ZoneSegmentStatus,
}

impl ZoneSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneSegmentStatus) -> bool {
        match self.status {
            ZoneSegmentStatus::Planned => {
                match next {
                    ZoneSegmentStatus::Scheduled | ZoneSegmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneSegmentStatus::Scheduled => {
                return next == ZoneSegmentStatus::Completed;
            }
            ZoneSegmentStatus::InProgress => {
                match next {
                    ZoneSegmentStatus::Completed | ZoneSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneSegmentStatus::Completed => {
                return next == ZoneSegmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "zone segment", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        (self.amount * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneSegment {
        ZoneSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            batch_size: self.batch_size,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a zone segment in status planned.
pub fn new_zone_segment(code: &str, title: &str) -> ZoneSegment {
    ZoneSegment {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        batch_size: 0,
        external_ref: "".to_string(),
        status: ZoneSegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_zone_segment("code-763", "title-555");
        assert_eq!(item.status, ZoneSegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_zone_segment("code-763", "title-555");
        if item.move_to(ZoneSegmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneSegmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_zone_segment("code-763", "title-555");
        item.status = ZoneSegmentStatus::Cancelled;
        assert!(
            item.move_to(ZoneSegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
