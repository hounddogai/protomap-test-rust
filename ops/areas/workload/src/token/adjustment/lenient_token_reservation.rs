use super::*;

use acme_kit::text;

/// Where a lenient token reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTokenReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientTokenReservationStatus {
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

impl std::fmt::Display for LenientTokenReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient token reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenReservation {
    pub code: String,
    pub label: String,
    pub name: String,
    pub priority: i64,
    pub title: String,
    pub status: LenientTokenReservationStatus,
}

impl LenientTokenReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTokenReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTokenReservationStatus) -> bool {
        match self.status {
            LenientTokenReservationStatus::New => {
                return next == LenientTokenReservationStatus::Queued;
            }
            LenientTokenReservationStatus::Queued => {
                match next {
                    LenientTokenReservationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTokenReservationStatus::Running => {
                match next {
                    LenientTokenReservationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTokenReservationStatus::Done => {
                match next {
                    LenientTokenReservationStatus::Failed => {
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

    pub fn move_to(&mut self, next: LenientTokenReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient token reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> LenientTokenReservation {
        LenientTokenReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            priority: self.priority,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient token reservation in status new.
pub fn new_lenient_token_reservation(code: &str, label: &str) -> LenientTokenReservation {
    LenientTokenReservation {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        priority: 0,
        title: "".to_string(),
        status: LenientTokenReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_token_reservation("code-234", "label-490");
        assert_eq!(item.status, LenientTokenReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_lenient_token_reservation("code-234", "label-490");
        if item.move_to(LenientTokenReservationStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientTokenReservationStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_token_reservation("code-234", "label-490");
        item.status = LenientTokenReservationStatus::Failed;
        assert!(
            item.move_to(LenientTokenReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
