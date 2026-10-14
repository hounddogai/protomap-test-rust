use super::*;

use acme_kit::text;

/// Where a reservation quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReservationQuotaStatus {
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

impl std::fmt::Display for ReservationQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a reservation quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationQuota {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub ceiling: f64,
    pub maximum: f64,
    pub status: ReservationQuotaStatus,
}

impl ReservationQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationQuotaStatus) -> bool {
        match self.status {
            ReservationQuotaStatus::Open => {
                return next == ReservationQuotaStatus::InReview || next == ReservationQuotaStatus::Reopened;
            }
            ReservationQuotaStatus::InReview => {
                match next {
                    ReservationQuotaStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationQuotaStatus::Resolved => {
                return next == ReservationQuotaStatus::Reopened || next == ReservationQuotaStatus::Archived;
            }
            ReservationQuotaStatus::Reopened => {
                match next {
                    ReservationQuotaStatus::Archived => {
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

    pub fn move_to(&mut self, next: ReservationQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ceiling + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationQuota {
        ReservationQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            ceiling: self.ceiling,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a reservation quota in status open.
pub fn new_reservation_quota(code: &str, label: &str) -> ReservationQuota {
    ReservationQuota {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        ceiling: 0.0,
        maximum: 0.0,
        status: ReservationQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_reservation_quota("code-715", "label-242");
        assert_eq!(item.status, ReservationQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_reservation_quota("code-715", "label-242");
        if item.move_to(ReservationQuotaStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationQuotaStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_reservation_quota("code-715", "label-242");
        item.status = ReservationQuotaStatus::Archived;
        assert!(
            item.move_to(ReservationQuotaStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
