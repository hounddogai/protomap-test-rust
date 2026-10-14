use super::*;

use acme_kit::text;

/// Where a token reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TokenReservationStatus {
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

impl std::fmt::Display for TokenReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a token reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenReservation {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub visible: bool,
    pub scope: String,
    pub score: f64,
    pub status: TokenReservationStatus,
}

impl TokenReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenReservationStatus) -> bool {
        match self.status {
            TokenReservationStatus::Draft => {
                match next {
                    TokenReservationStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenReservationStatus::Pending => {
                return next == TokenReservationStatus::Active || next == TokenReservationStatus::Suspended;
            }
            TokenReservationStatus::Active => {
                return next == TokenReservationStatus::Suspended || next == TokenReservationStatus::Closed;
            }
            TokenReservationStatus::Suspended => {
                return next == TokenReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> TokenReservation {
        TokenReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            visible: self.visible,
            scope: self.scope.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a token reservation in status draft.
pub fn new_token_reservation(code: &str, title: &str) -> TokenReservation {
    TokenReservation {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        visible: false,
        scope: "".to_string(),
        score: 0.0,
        status: TokenReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_token_reservation("code-369", "title-702");
        assert_eq!(item.status, TokenReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_token_reservation("code-369", "title-702");
        if item.move_to(TokenReservationStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenReservationStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_token_reservation("code-369", "title-702");
        item.status = TokenReservationStatus::Closed;
        assert!(
            item.move_to(TokenReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
