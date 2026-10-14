use super::*;

use acme_kit::text;

/// Where a pending grant metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingGrantMetricStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingGrantMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingGrantMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending grant metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingGrantMetric {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub capacity: i64,
    pub country: String,
    pub timezone: String,
    pub amount: f64,
    pub sequence: i64,
    pub locale: String,
    pub batch_size: i64,
    pub status: PendingGrantMetricStatus,
}

impl PendingGrantMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingGrantMetricStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingGrantMetricStatus) -> bool {
        match self.status {
            PendingGrantMetricStatus::Requested => {
                return next == PendingGrantMetricStatus::Revoked;
            }
            PendingGrantMetricStatus::Granted => {
                match next {
                    PendingGrantMetricStatus::Revoked | PendingGrantMetricStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingGrantMetricStatus::Revoked => {
                match next {
                    PendingGrantMetricStatus::Expired => {
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

    pub fn move_to(&mut self, next: PendingGrantMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending grant metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.amount + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingGrantMetric {
        PendingGrantMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            capacity: self.capacity,
            country: self.country.clone(),
            timezone: self.timezone.clone(),
            amount: self.amount,
            sequence: self.sequence,
            locale: self.locale.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a pending grant metric in status requested.
pub fn new_pending_grant_metric(code: &str, title: &str) -> PendingGrantMetric {
    PendingGrantMetric {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        capacity: 0,
        country: "".to_string(),
        timezone: "".to_string(),
        amount: 0.0,
        sequence: 0,
        locale: "".to_string(),
        batch_size: 0,
        status: PendingGrantMetricStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_grant_metric("code-594", "title-324");
        assert_eq!(item.status, PendingGrantMetricStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_pending_grant_metric("code-594", "title-324");
        if item.move_to(PendingGrantMetricStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingGrantMetricStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_grant_metric("code-594", "title-324");
        item.status = PendingGrantMetricStatus::Expired;
        assert!(
            item.move_to(PendingGrantMetricStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
