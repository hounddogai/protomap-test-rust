use super::*;

use acme_kit::text;

/// Where a primary slo segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySloSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimarySloSegmentStatus {
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

impl std::fmt::Display for PrimarySloSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary slo segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloSegment {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub duration_minutes: i64,
    pub timezone: String,
    pub channel: String,
    pub host: String,
    pub issuer: String,
    pub fingerprint: String,
    pub minimum: f64,
    pub principal: String,
    pub status: PrimarySloSegmentStatus,
}

impl PrimarySloSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySloSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySloSegmentStatus) -> bool {
        match self.status {
            PrimarySloSegmentStatus::Planned => {
                return next == PrimarySloSegmentStatus::Scheduled || next == PrimarySloSegmentStatus::Cancelled;
            }
            PrimarySloSegmentStatus::Scheduled => {
                match next {
                    PrimarySloSegmentStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySloSegmentStatus::InProgress => {
                return next == PrimarySloSegmentStatus::Cancelled;
            }
            PrimarySloSegmentStatus::Completed => {
                return next == PrimarySloSegmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimarySloSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary slo segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.minimum + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySloSegment {
        PrimarySloSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            duration_minutes: self.duration_minutes,
            timezone: self.timezone.clone(),
            channel: self.channel.clone(),
            host: self.host.clone(),
            issuer: self.issuer.clone(),
            fingerprint: self.fingerprint.clone(),
            minimum: self.minimum,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a primary slo segment in status planned.
pub fn new_primary_slo_segment(code: &str, title: &str) -> PrimarySloSegment {
    PrimarySloSegment {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        duration_minutes: 0,
        timezone: "".to_string(),
        channel: "".to_string(),
        host: "".to_string(),
        issuer: "".to_string(),
        fingerprint: "".to_string(),
        minimum: 0.0,
        principal: "".to_string(),
        status: PrimarySloSegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_slo_segment("code-290", "title-315");
        assert_eq!(item.status, PrimarySloSegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_primary_slo_segment("code-290", "title-315");
        if item.move_to(PrimarySloSegmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySloSegmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_slo_segment("code-290", "title-315");
        item.status = PrimarySloSegmentStatus::Cancelled;
        assert!(
            item.move_to(PrimarySloSegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
