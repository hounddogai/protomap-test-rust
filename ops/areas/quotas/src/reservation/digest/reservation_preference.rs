use super::*;

use acme_kit::text;

/// Where a reservation preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationPreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ReservationPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ReservationPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One reservation preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationPreference {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub scope: String,
    pub margin: f64,
    pub approved: bool,
    pub quota_units: i64,
    pub channel: String,
    pub ratio: f64,
    pub status: ReservationPreferenceStatus,
}

impl ReservationPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationPreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationPreferenceStatus) -> bool {
        match self.status {
            ReservationPreferenceStatus::Requested => {
                match next {
                    ReservationPreferenceStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationPreferenceStatus::Granted => {
                return next == ReservationPreferenceStatus::Expired;
            }
            ReservationPreferenceStatus::Revoked => {
                return next == ReservationPreferenceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ratio + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationPreference {
        ReservationPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            scope: self.scope.clone(),
            margin: self.margin,
            approved: self.approved,
            quota_units: self.quota_units,
            channel: self.channel.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a reservation preference in status requested.
pub fn new_reservation_preference(code: &str, label: &str) -> ReservationPreference {
    ReservationPreference {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        scope: "".to_string(),
        margin: 0.0,
        approved: false,
        quota_units: 0,
        channel: "".to_string(),
        ratio: 0.0,
        status: ReservationPreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_reservation_preference("code-646", "label-718");
        assert_eq!(item.status, ReservationPreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_reservation_preference("code-646", "label-718");
        if item.move_to(ReservationPreferenceStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationPreferenceStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_reservation_preference("code-646", "label-718");
        item.status = ReservationPreferenceStatus::Expired;
        assert!(
            item.move_to(ReservationPreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
