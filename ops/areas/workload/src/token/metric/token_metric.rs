use super::*;

use acme_kit::text;

/// Where a token metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TokenMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TokenMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a token metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenMetric {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub name: String,
    pub budget: f64,
    pub updated_at: i64,
    pub maximum: f64,
    pub duration_minutes: i64,
    pub currency: String,
    pub status: TokenMetricStatus,
}

impl TokenMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenMetricStatus) -> bool {
        match self.status {
            TokenMetricStatus::Created => {
                return next == TokenMetricStatus::Published || next == TokenMetricStatus::Retired;
            }
            TokenMetricStatus::Validated => {
                return next == TokenMetricStatus::Published || next == TokenMetricStatus::Retired;
            }
            TokenMetricStatus::Published => {
                return next == TokenMetricStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token metric {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.maximum * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenMetric {
        TokenMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            name: self.name.clone(),
            budget: self.budget,
            updated_at: self.updated_at,
            maximum: self.maximum,
            duration_minutes: self.duration_minutes,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a token metric in status created.
pub fn new_token_metric(code: &str, label: &str) -> TokenMetric {
    TokenMetric {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        name: "".to_string(),
        budget: 0.0,
        updated_at: 0,
        maximum: 0.0,
        duration_minutes: 0,
        currency: "".to_string(),
        status: TokenMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_token_metric("code-247", "label-537");
        assert_eq!(item.status, TokenMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_token_metric("code-247", "label-537");
        if item.move_to(TokenMetricStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenMetricStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_token_metric("code-247", "label-537");
        item.status = TokenMetricStatus::Retired;
        assert!(
            item.move_to(TokenMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
