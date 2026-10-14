use super::*;

use acme_kit::text;

/// Where a measurement profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MeasurementProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for MeasurementProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementProfile {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub country: String,
    pub threshold: f64,
    pub timezone: String,
    pub grace_days: i64,
    pub currency: String,
    pub pinned: bool,
    pub status: MeasurementProfileStatus,
}

impl MeasurementProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementProfileStatus) -> bool {
        match self.status {
            MeasurementProfileStatus::New => {
                return next == MeasurementProfileStatus::Queued || next == MeasurementProfileStatus::Running;
            }
            MeasurementProfileStatus::Queued => {
                match next {
                    MeasurementProfileStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementProfileStatus::Running => {
                return next == MeasurementProfileStatus::Done;
            }
            MeasurementProfileStatus::Done => {
                return next == MeasurementProfileStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "measurement profile", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.threshold - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementProfile {
        MeasurementProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            country: self.country.clone(),
            threshold: self.threshold,
            timezone: self.timezone.clone(),
            grace_days: self.grace_days,
            currency: self.currency.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a measurement profile in status new.
pub fn new_measurement_profile(code: &str, label: &str) -> MeasurementProfile {
    MeasurementProfile {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        country: "".to_string(),
        threshold: 0.0,
        timezone: "".to_string(),
        grace_days: 0,
        currency: "".to_string(),
        pinned: false,
        status: MeasurementProfileStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_measurement_profile("code-103", "label-705");
        assert_eq!(item.status, MeasurementProfileStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_measurement_profile("code-103", "label-705");
        if item.move_to(MeasurementProfileStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementProfileStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_measurement_profile("code-103", "label-705");
        item.status = MeasurementProfileStatus::Failed;
        assert!(
            item.move_to(MeasurementProfileStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
