use super::*;

use acme_kit::text;

/// Where a quote claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl QuoteClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for QuoteClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One quote claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteClaim {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub country: String,
    pub visible: bool,
    pub scope: String,
    pub burst: i64,
    pub key_id: String,
    pub status: QuoteClaimStatus,
}

impl QuoteClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteClaimStatus) -> bool {
        match self.status {
            QuoteClaimStatus::Planned => {
                return next == QuoteClaimStatus::Completed;
            }
            QuoteClaimStatus::Scheduled => {
                match next {
                    QuoteClaimStatus::InProgress | QuoteClaimStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteClaimStatus::InProgress => {
                return next == QuoteClaimStatus::Completed;
            }
            QuoteClaimStatus::Completed => {
                return next == QuoteClaimStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.expires_at as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteClaim {
        QuoteClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            country: self.country.clone(),
            visible: self.visible,
            scope: self.scope.clone(),
            burst: self.burst,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a quote claim in status planned.
pub fn new_quote_claim(code: &str, label: &str) -> QuoteClaim {
    QuoteClaim {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        country: "".to_string(),
        visible: false,
        scope: "".to_string(),
        burst: 0,
        key_id: "".to_string(),
        status: QuoteClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_quote_claim("code-780", "label-629");
        assert_eq!(item.status, QuoteClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_quote_claim("code-780", "label-629");
        if item.move_to(QuoteClaimStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteClaimStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_quote_claim("code-780", "label-629");
        item.status = QuoteClaimStatus::Cancelled;
        assert!(
            item.move_to(QuoteClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
