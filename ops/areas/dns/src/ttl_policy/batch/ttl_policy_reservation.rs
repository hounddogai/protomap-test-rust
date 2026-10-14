use super::*;

use acme_kit::text;

/// Where a ttl policy reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TtlPolicyReservationStatus {
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

impl std::fmt::Display for TtlPolicyReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a ttl policy reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyReservation {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub checksum: String,
    pub description: String,
    pub external_ref: String,
    pub issuer: String,
    pub currency: String,
    pub summary: String,
    pub status: TtlPolicyReservationStatus,
}

impl TtlPolicyReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyReservationStatus) -> bool {
        match self.status {
            TtlPolicyReservationStatus::New => {
                match next {
                    TtlPolicyReservationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyReservationStatus::Queued => {
                return next == TtlPolicyReservationStatus::Done || next == TtlPolicyReservationStatus::Failed;
            }
            TtlPolicyReservationStatus::Running => {
                return next == TtlPolicyReservationStatus::Done;
            }
            TtlPolicyReservationStatus::Done => {
                match next {
                    TtlPolicyReservationStatus::Failed => {
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

    pub fn move_to(&mut self, next: TtlPolicyReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> TtlPolicyReservation {
        TtlPolicyReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            checksum: self.checksum.clone(),
            description: self.description.clone(),
            external_ref: self.external_ref.clone(),
            issuer: self.issuer.clone(),
            currency: self.currency.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a ttl policy reservation in status new.
pub fn new_ttl_policy_reservation(code: &str, title: &str) -> TtlPolicyReservation {
    TtlPolicyReservation {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        checksum: "".to_string(),
        description: "".to_string(),
        external_ref: "".to_string(),
        issuer: "".to_string(),
        currency: "".to_string(),
        summary: "".to_string(),
        status: TtlPolicyReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_ttl_policy_reservation("code-873", "title-622");
        assert_eq!(item.status, TtlPolicyReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_ttl_policy_reservation("code-873", "title-622");
        if item.move_to(TtlPolicyReservationStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyReservationStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_ttl_policy_reservation("code-873", "title-622");
        item.status = TtlPolicyReservationStatus::Failed;
        assert!(
            item.move_to(TtlPolicyReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
