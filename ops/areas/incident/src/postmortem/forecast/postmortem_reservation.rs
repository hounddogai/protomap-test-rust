use super::*;

use acme_kit::text;

/// Where a postmortem reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PostmortemReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PostmortemReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemReservation {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub batch_size: i64,
    pub contact_email: String,
    pub port: i64,
    pub phone: String,
    pub approved: bool,
    pub notes: String,
    pub minimum: f64,
    pub cipher: String,
    pub status: PostmortemReservationStatus,
}

impl PostmortemReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemReservationStatus) -> bool {
        match self.status {
            PostmortemReservationStatus::Requested => {
                return next == PostmortemReservationStatus::Granted;
            }
            PostmortemReservationStatus::Granted => {
                return next == PostmortemReservationStatus::Revoked;
            }
            PostmortemReservationStatus::Revoked => {
                match next {
                    PostmortemReservationStatus::Expired => {
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

    pub fn move_to(&mut self, next: PostmortemReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.port as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemReservation {
        PostmortemReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            batch_size: self.batch_size,
            contact_email: self.contact_email.clone(),
            port: self.port,
            phone: self.phone.clone(),
            approved: self.approved,
            notes: self.notes.clone(),
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a postmortem reservation in status requested.
pub fn new_postmortem_reservation(code: &str, label: &str) -> PostmortemReservation {
    PostmortemReservation {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        batch_size: 0,
        contact_email: "".to_string(),
        port: 0,
        phone: "".to_string(),
        approved: false,
        notes: "".to_string(),
        minimum: 0.0,
        cipher: "".to_string(),
        status: PostmortemReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_postmortem_reservation("code-929", "label-573");
        assert_eq!(item.status, PostmortemReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_postmortem_reservation("code-929", "label-573");
        if item.move_to(PostmortemReservationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemReservationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_postmortem_reservation("code-929", "label-573");
        item.status = PostmortemReservationStatus::Expired;
        assert!(
            item.move_to(PostmortemReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
