use super::*;

use acme_kit::text;

/// Where a timeline agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TimelineAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TimelineAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One timeline agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAgreement {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub ttl_seconds: i64,
    pub country: String,
    pub grace_days: i64,
    pub capacity: i64,
    pub phone: String,
    pub unit: String,
    pub region: String,
    pub version: i64,
    pub status: TimelineAgreementStatus,
}

impl TimelineAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineAgreementStatus) -> bool {
        match self.status {
            TimelineAgreementStatus::Idle => {
                match next {
                    TimelineAgreementStatus::Ready | TimelineAgreementStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineAgreementStatus::Warming => {
                match next {
                    TimelineAgreementStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineAgreementStatus::Ready => {
                return next == TimelineAgreementStatus::Draining || next == TimelineAgreementStatus::Stopped;
            }
            TimelineAgreementStatus::Draining => {
                match next {
                    TimelineAgreementStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TimelineAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "timeline agreement", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.grace_days as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineAgreement {
        TimelineAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            ttl_seconds: self.ttl_seconds,
            country: self.country.clone(),
            grace_days: self.grace_days,
            capacity: self.capacity,
            phone: self.phone.clone(),
            unit: self.unit.clone(),
            region: self.region.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a timeline agreement in status idle.
pub fn new_timeline_agreement(code: &str, title: &str) -> TimelineAgreement {
    TimelineAgreement {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        ttl_seconds: 0,
        country: "".to_string(),
        grace_days: 0,
        capacity: 0,
        phone: "".to_string(),
        unit: "".to_string(),
        region: "".to_string(),
        version: 0,
        status: TimelineAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_timeline_agreement("code-968", "title-2");
        assert_eq!(item.status, TimelineAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_timeline_agreement("code-968", "title-2");
        if item.move_to(TimelineAgreementStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineAgreementStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_timeline_agreement("code-968", "title-2");
        item.status = TimelineAgreementStatus::Stopped;
        assert!(
            item.move_to(TimelineAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
