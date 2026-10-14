use super::*;

use acme_kit::text;

/// Where a lenient ttl policy reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTtlPolicyReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientTtlPolicyReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LenientTtlPolicyReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient ttl policy reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTtlPolicyReservation {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub timezone: String,
    pub fingerprint: String,
    pub score: f64,
    pub quota_units: i64,
    pub status: LenientTtlPolicyReservationStatus,
}

impl LenientTtlPolicyReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTtlPolicyReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTtlPolicyReservationStatus) -> bool {
        match self.status {
            LenientTtlPolicyReservationStatus::Draft => {
                return next == LenientTtlPolicyReservationStatus::Closed;
            }
            LenientTtlPolicyReservationStatus::Pending => {
                return next == LenientTtlPolicyReservationStatus::Active
                    || next == LenientTtlPolicyReservationStatus::Suspended;
            }
            LenientTtlPolicyReservationStatus::Active => {
                return next == LenientTtlPolicyReservationStatus::Suspended
                    || next == LenientTtlPolicyReservationStatus::Closed;
            }
            LenientTtlPolicyReservationStatus::Suspended => {
                return next == LenientTtlPolicyReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientTtlPolicyReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient ttl policy reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.quota_units as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTtlPolicyReservation {
        LenientTtlPolicyReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            timezone: self.timezone.clone(),
            fingerprint: self.fingerprint.clone(),
            score: self.score,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient ttl policy reservation in status draft.
pub fn new_lenient_ttl_policy_reservation(code: &str, label: &str) -> LenientTtlPolicyReservation {
    LenientTtlPolicyReservation {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        timezone: "".to_string(),
        fingerprint: "".to_string(),
        score: 0.0,
        quota_units: 0,
        status: LenientTtlPolicyReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_ttl_policy_reservation("code-210", "label-555");
        assert_eq!(item.status, LenientTtlPolicyReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_lenient_ttl_policy_reservation("code-210", "label-555");
        if item.move_to(LenientTtlPolicyReservationStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientTtlPolicyReservationStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_ttl_policy_reservation("code-210", "label-555");
        item.status = LenientTtlPolicyReservationStatus::Closed;
        assert!(
            item.move_to(LenientTtlPolicyReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
