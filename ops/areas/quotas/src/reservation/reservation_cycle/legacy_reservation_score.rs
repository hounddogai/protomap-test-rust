use super::*;

use acme_kit::text;

/// Where a legacy reservation score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyReservationScoreStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyReservationScoreStatus {
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

impl std::fmt::Display for LegacyReservationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy reservation score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationScore {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub contact_email: String,
    pub limit: i64,
    pub batch_size: i64,
    pub weight: f64,
    pub grace_days: i64,
    pub issuer: String,
    pub name: String,
    pub principal: String,
    pub status: LegacyReservationScoreStatus,
}

impl LegacyReservationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyReservationScoreStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyReservationScoreStatus) -> bool {
        match self.status {
            LegacyReservationScoreStatus::Idle => {
                match next {
                    LegacyReservationScoreStatus::Ready | LegacyReservationScoreStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyReservationScoreStatus::Warming => {
                match next {
                    LegacyReservationScoreStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyReservationScoreStatus::Ready => {
                return next == LegacyReservationScoreStatus::Draining;
            }
            LegacyReservationScoreStatus::Draining => {
                return next == LegacyReservationScoreStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyReservationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy reservation score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        (self.weight * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyReservationScore {
        LegacyReservationScore {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            contact_email: self.contact_email.clone(),
            limit: self.limit,
            batch_size: self.batch_size,
            weight: self.weight,
            grace_days: self.grace_days,
            issuer: self.issuer.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy reservation score in status idle.
pub fn new_legacy_reservation_score(code: &str, title: &str) -> LegacyReservationScore {
    LegacyReservationScore {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        contact_email: "".to_string(),
        limit: 0,
        batch_size: 0,
        weight: 0.0,
        grace_days: 0,
        issuer: "".to_string(),
        name: "".to_string(),
        principal: "".to_string(),
        status: LegacyReservationScoreStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_reservation_score("code-309", "title-786");
        assert_eq!(item.status, LegacyReservationScoreStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_legacy_reservation_score("code-309", "title-786");
        if item.move_to(LegacyReservationScoreStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyReservationScoreStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_reservation_score("code-309", "title-786");
        item.status = LegacyReservationScoreStatus::Stopped;
        assert!(
            item.move_to(LegacyReservationScoreStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
