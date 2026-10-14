use super::*;

use acme_kit::text;

/// Where a seasonal reservation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalReservationProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalReservationProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for SeasonalReservationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal reservation profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationProfile {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub ratio: f64,
    pub audience: String,
    pub status: SeasonalReservationProfileStatus,
}

impl SeasonalReservationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalReservationProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalReservationProfileStatus) -> bool {
        match self.status {
            SeasonalReservationProfileStatus::Open => {
                match next {
                    SeasonalReservationProfileStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalReservationProfileStatus::InReview => {
                match next {
                    SeasonalReservationProfileStatus::Resolved | SeasonalReservationProfileStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalReservationProfileStatus::Resolved => {
                return next == SeasonalReservationProfileStatus::Reopened
                    || next == SeasonalReservationProfileStatus::Archived;
            }
            SeasonalReservationProfileStatus::Reopened => {
                match next {
                    SeasonalReservationProfileStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalReservationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal reservation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal reservation profile", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ratio
    }

    pub fn copy(&self) -> SeasonalReservationProfile {
        SeasonalReservationProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            ratio: self.ratio,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal reservation profile in status open.
pub fn new_seasonal_reservation_profile(code: &str, label: &str) -> SeasonalReservationProfile {
    SeasonalReservationProfile {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        ratio: 0.0,
        audience: "".to_string(),
        status: SeasonalReservationProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_reservation_profile("code-528", "label-143");
        assert_eq!(item.status, SeasonalReservationProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_seasonal_reservation_profile("code-528", "label-143");
        if item.move_to(SeasonalReservationProfileStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalReservationProfileStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_reservation_profile("code-528", "label-143");
        item.status = SeasonalReservationProfileStatus::Archived;
        assert!(
            item.move_to(SeasonalReservationProfileStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
