use super::*;

use acme_kit::text;

/// Where a freeze summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FreezeSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FreezeSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSummary {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub host: String,
    pub quota_units: i64,
    pub maximum: f64,
    pub budget: f64,
    pub floor_value: f64,
    pub fingerprint: String,
    pub discount: f64,
    pub batch_size: i64,
    pub status: FreezeSummaryStatus,
}

impl FreezeSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeSummaryStatus) -> bool {
        match self.status {
            FreezeSummaryStatus::Requested => {
                return next == FreezeSummaryStatus::Granted || next == FreezeSummaryStatus::Expired;
            }
            FreezeSummaryStatus::Granted => {
                return next == FreezeSummaryStatus::Revoked;
            }
            FreezeSummaryStatus::Revoked => {
                return next == FreezeSummaryStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.floor_value + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeSummary {
        FreezeSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            host: self.host.clone(),
            quota_units: self.quota_units,
            maximum: self.maximum,
            budget: self.budget,
            floor_value: self.floor_value,
            fingerprint: self.fingerprint.clone(),
            discount: self.discount,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a freeze summary in status requested.
pub fn new_freeze_summary(code: &str, label: &str) -> FreezeSummary {
    FreezeSummary {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        host: "".to_string(),
        quota_units: 0,
        maximum: 0.0,
        budget: 0.0,
        floor_value: 0.0,
        fingerprint: "".to_string(),
        discount: 0.0,
        batch_size: 0,
        status: FreezeSummaryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_freeze_summary("code-562", "label-440");
        assert_eq!(item.status, FreezeSummaryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_freeze_summary("code-562", "label-440");
        if item.move_to(FreezeSummaryStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeSummaryStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_freeze_summary("code-562", "label-440");
        item.status = FreezeSummaryStatus::Expired;
        assert!(
            item.move_to(FreezeSummaryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
