use super::*;

use acme_kit::text;

/// Where a capped reservation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedReservationVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedReservationVarianceStatus {
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

impl std::fmt::Display for CappedReservationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped reservation variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationVariance {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub amount: f64,
    pub pinned: bool,
    pub phone: String,
    pub currency: String,
    pub issuer: String,
    pub margin: f64,
    pub status: CappedReservationVarianceStatus,
}

impl CappedReservationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedReservationVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedReservationVarianceStatus) -> bool {
        match self.status {
            CappedReservationVarianceStatus::New => {
                match next {
                    CappedReservationVarianceStatus::Running | CappedReservationVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedReservationVarianceStatus::Queued => {
                match next {
                    CappedReservationVarianceStatus::Running | CappedReservationVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedReservationVarianceStatus::Running => {
                return next == CappedReservationVarianceStatus::Failed;
            }
            CappedReservationVarianceStatus::Done => {
                return next == CappedReservationVarianceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedReservationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped reservation variance {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.margin + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedReservationVariance {
        CappedReservationVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            amount: self.amount,
            pinned: self.pinned,
            phone: self.phone.clone(),
            currency: self.currency.clone(),
            issuer: self.issuer.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped reservation variance in status new.
pub fn new_capped_reservation_variance(code: &str, title: &str) -> CappedReservationVariance {
    CappedReservationVariance {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        amount: 0.0,
        pinned: false,
        phone: "".to_string(),
        currency: "".to_string(),
        issuer: "".to_string(),
        margin: 0.0,
        status: CappedReservationVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_reservation_variance("code-471", "title-141");
        assert_eq!(item.status, CappedReservationVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_capped_reservation_variance("code-471", "title-141");
        if item.move_to(CappedReservationVarianceStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedReservationVarianceStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_reservation_variance("code-471", "title-141");
        item.status = CappedReservationVarianceStatus::Failed;
        assert!(
            item.move_to(CappedReservationVarianceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
