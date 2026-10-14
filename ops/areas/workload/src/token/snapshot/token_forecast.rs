use super::*;

use acme_kit::text;

/// Where a token forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TokenForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TokenForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a token forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenForecast {
    pub code: String,
    pub title: String,
    pub label: String,
    pub floor_value: f64,
    pub visible: bool,
    pub channel: String,
    pub status: TokenForecastStatus,
}

impl TokenForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenForecastStatus) -> bool {
        match self.status {
            TokenForecastStatus::Proposed => {
                match next {
                    TokenForecastStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenForecastStatus::Approved => {
                return next == TokenForecastStatus::Rejected || next == TokenForecastStatus::Withdrawn;
            }
            TokenForecastStatus::Rejected => {
                return next == TokenForecastStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> TokenForecast {
        TokenForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            visible: self.visible,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a token forecast in status proposed.
pub fn new_token_forecast(code: &str, title: &str) -> TokenForecast {
    TokenForecast {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        floor_value: 0.0,
        visible: false,
        channel: "".to_string(),
        status: TokenForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_token_forecast("code-325", "title-841");
        assert_eq!(item.status, TokenForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_token_forecast("code-325", "title-841");
        if item.move_to(TokenForecastStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenForecastStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_token_forecast("code-325", "title-841");
        item.status = TokenForecastStatus::Withdrawn;
        assert!(
            item.move_to(TokenForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
