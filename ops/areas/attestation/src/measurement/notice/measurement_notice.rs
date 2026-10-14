use super::*;

use acme_kit::text;

/// Where a measurement notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementNoticeStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MeasurementNoticeStatus {
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

impl std::fmt::Display for MeasurementNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementNotice {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub category: String,
    pub owner_id: String,
    pub ratio: f64,
    pub updated_at: i64,
    pub duration_minutes: i64,
    pub burst: i64,
    pub status: MeasurementNoticeStatus,
}

impl MeasurementNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementNoticeStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementNoticeStatus) -> bool {
        match self.status {
            MeasurementNoticeStatus::Idle => {
                return next == MeasurementNoticeStatus::Warming || next == MeasurementNoticeStatus::Draining;
            }
            MeasurementNoticeStatus::Warming => {
                return next == MeasurementNoticeStatus::Draining;
            }
            MeasurementNoticeStatus::Ready => {
                return next == MeasurementNoticeStatus::Draining || next == MeasurementNoticeStatus::Stopped;
            }
            MeasurementNoticeStatus::Draining => {
                match next {
                    MeasurementNoticeStatus::Stopped => {
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

    pub fn move_to(&mut self, next: MeasurementNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "measurement notice", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.burst as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementNotice {
        MeasurementNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            category: self.category.clone(),
            owner_id: self.owner_id.clone(),
            ratio: self.ratio,
            updated_at: self.updated_at,
            duration_minutes: self.duration_minutes,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a measurement notice in status idle.
pub fn new_measurement_notice(code: &str, title: &str) -> MeasurementNotice {
    MeasurementNotice {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        category: "".to_string(),
        owner_id: "".to_string(),
        ratio: 0.0,
        updated_at: 0,
        duration_minutes: 0,
        burst: 0,
        status: MeasurementNoticeStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_measurement_notice("code-447", "title-159");
        assert_eq!(item.status, MeasurementNoticeStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_measurement_notice("code-447", "title-159");
        if item.move_to(MeasurementNoticeStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementNoticeStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_measurement_notice("code-447", "title-159");
        item.status = MeasurementNoticeStatus::Stopped;
        assert!(
            item.move_to(MeasurementNoticeStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
