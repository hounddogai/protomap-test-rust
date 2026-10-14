use super::*;

use acme_kit::text;

/// Where a tiered reservation summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredReservationSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredReservationSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredReservationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered reservation summary as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationSummary {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub cost: f64,
    pub issuer: String,
    pub visible: bool,
    pub priority: i64,
    pub currency: String,
    pub archived: bool,
    pub status: TieredReservationSummaryStatus,
}

impl TieredReservationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredReservationSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredReservationSummaryStatus) -> bool {
        match self.status {
            TieredReservationSummaryStatus::Requested => {
                return next == TieredReservationSummaryStatus::Revoked
                    || next == TieredReservationSummaryStatus::Expired;
            }
            TieredReservationSummaryStatus::Granted => {
                match next {
                    TieredReservationSummaryStatus::Revoked | TieredReservationSummaryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredReservationSummaryStatus::Revoked => {
                return next == TieredReservationSummaryStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredReservationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered reservation summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered reservation summary", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.priority as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredReservationSummary {
        TieredReservationSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            cost: self.cost,
            issuer: self.issuer.clone(),
            visible: self.visible,
            priority: self.priority,
            currency: self.currency.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a tiered reservation summary in status requested.
pub fn new_tiered_reservation_summary(code: &str, title: &str) -> TieredReservationSummary {
    TieredReservationSummary {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        cost: 0.0,
        issuer: "".to_string(),
        visible: false,
        priority: 0,
        currency: "".to_string(),
        archived: false,
        status: TieredReservationSummaryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_tiered_reservation_summary("code-263", "title-864");
        assert_eq!(item.status, TieredReservationSummaryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_tiered_reservation_summary("code-263", "title-864");
        if item.move_to(TieredReservationSummaryStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredReservationSummaryStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_tiered_reservation_summary("code-263", "title-864");
        item.status = TieredReservationSummaryStatus::Expired;
        assert!(
            item.move_to(TieredReservationSummaryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
