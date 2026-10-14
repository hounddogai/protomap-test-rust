use super::*;

use acme_kit::text;

/// Where a primary ttl policy reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTtlPolicyReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryTtlPolicyReservationStatus {
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

impl std::fmt::Display for PrimaryTtlPolicyReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary ttl policy reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyReservation {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub tags: Vec<String>,
    pub country: String,
    pub budget: f64,
    pub rate: f64,
    pub capacity: i64,
    pub status: PrimaryTtlPolicyReservationStatus,
}

impl PrimaryTtlPolicyReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTtlPolicyReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTtlPolicyReservationStatus) -> bool {
        match self.status {
            PrimaryTtlPolicyReservationStatus::New => {
                match next {
                    PrimaryTtlPolicyReservationStatus::Done | PrimaryTtlPolicyReservationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTtlPolicyReservationStatus::Queued => {
                match next {
                    PrimaryTtlPolicyReservationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTtlPolicyReservationStatus::Running => {
                return next == PrimaryTtlPolicyReservationStatus::Failed;
            }
            PrimaryTtlPolicyReservationStatus::Done => {
                return next == PrimaryTtlPolicyReservationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryTtlPolicyReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary ttl policy reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_total(&self) -> f64 {
        (self.rate * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryTtlPolicyReservation {
        PrimaryTtlPolicyReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            tags: self.tags.clone(),
            country: self.country.clone(),
            budget: self.budget,
            rate: self.rate,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a primary ttl policy reservation in status new.
pub fn new_primary_ttl_policy_reservation(code: &str, title: &str) -> PrimaryTtlPolicyReservation {
    PrimaryTtlPolicyReservation {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        tags: Vec::new(),
        country: "".to_string(),
        budget: 0.0,
        rate: 0.0,
        capacity: 0,
        status: PrimaryTtlPolicyReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_ttl_policy_reservation("code-177", "title-730");
        assert_eq!(item.status, PrimaryTtlPolicyReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_primary_ttl_policy_reservation("code-177", "title-730");
        if item.move_to(PrimaryTtlPolicyReservationStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryTtlPolicyReservationStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_ttl_policy_reservation("code-177", "title-730");
        item.status = PrimaryTtlPolicyReservationStatus::Failed;
        assert!(
            item.move_to(PrimaryTtlPolicyReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
