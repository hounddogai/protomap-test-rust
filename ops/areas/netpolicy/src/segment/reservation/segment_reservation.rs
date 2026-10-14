use super::*;

use acme_kit::text;

/// Where a segment reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SegmentReservationStatus {
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

impl std::fmt::Display for SegmentReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment reservation as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReservation {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub retries: i64,
    pub timezone: String,
    pub status: SegmentReservationStatus,
}

impl SegmentReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentReservationStatus) -> bool {
        match self.status {
            SegmentReservationStatus::Idle => {
                match next {
                    SegmentReservationStatus::Warming | SegmentReservationStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentReservationStatus::Warming => {
                match next {
                    SegmentReservationStatus::Ready | SegmentReservationStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentReservationStatus::Ready => {
                match next {
                    SegmentReservationStatus::Draining | SegmentReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentReservationStatus::Draining => {
                match next {
                    SegmentReservationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SegmentReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "segment reservation", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.starts_at as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentReservation {
        SegmentReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            retries: self.retries,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a segment reservation in status idle.
pub fn new_segment_reservation(code: &str, title: &str) -> SegmentReservation {
    SegmentReservation {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        retries: 0,
        timezone: "".to_string(),
        status: SegmentReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_segment_reservation("code-70", "title-77");
        assert_eq!(item.status, SegmentReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_segment_reservation("code-70", "title-77");
        if item.move_to(SegmentReservationStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentReservationStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_segment_reservation("code-70", "title-77");
        item.status = SegmentReservationStatus::Stopped;
        assert!(
            item.move_to(SegmentReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
