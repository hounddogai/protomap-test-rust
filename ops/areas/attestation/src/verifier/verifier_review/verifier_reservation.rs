use super::*;

use acme_kit::text;

/// Where a verifier reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl VerifierReservationStatus {
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

impl std::fmt::Display for VerifierReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierReservation {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub region: String,
    pub channel: String,
    pub minimum: f64,
    pub score: f64,
    pub source: String,
    pub starts_at: i64,
    pub created_at: i64,
    pub issuer: String,
    pub status: VerifierReservationStatus,
}

impl VerifierReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierReservationStatus) -> bool {
        match self.status {
            VerifierReservationStatus::New => {
                match next {
                    VerifierReservationStatus::Queued | VerifierReservationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierReservationStatus::Queued => {
                return next == VerifierReservationStatus::Done || next == VerifierReservationStatus::Failed;
            }
            VerifierReservationStatus::Running => {
                match next {
                    VerifierReservationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierReservationStatus::Done => {
                return next == VerifierReservationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier reservation {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.score - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierReservation {
        VerifierReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            region: self.region.clone(),
            channel: self.channel.clone(),
            minimum: self.minimum,
            score: self.score,
            source: self.source.clone(),
            starts_at: self.starts_at,
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a verifier reservation in status new.
pub fn new_verifier_reservation(code: &str, name: &str) -> VerifierReservation {
    VerifierReservation {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        region: "".to_string(),
        channel: "".to_string(),
        minimum: 0.0,
        score: 0.0,
        source: "".to_string(),
        starts_at: 0,
        created_at: 0,
        issuer: "".to_string(),
        status: VerifierReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_verifier_reservation("code-356", "name-999");
        assert_eq!(item.status, VerifierReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_verifier_reservation("code-356", "name-999");
        if item.move_to(VerifierReservationStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierReservationStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_verifier_reservation("code-356", "name-999");
        item.status = VerifierReservationStatus::Failed;
        assert!(
            item.move_to(VerifierReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
