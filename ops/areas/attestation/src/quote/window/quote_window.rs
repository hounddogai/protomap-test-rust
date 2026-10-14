use super::*;

use acme_kit::text;

/// Where a quote window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteWindowStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl QuoteWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for QuoteWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteWindow {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub description: String,
    pub retries: i64,
    pub unit: String,
    pub ratio: f64,
    pub locale: String,
    pub margin: f64,
    pub sequence: i64,
    pub status: QuoteWindowStatus,
}

impl QuoteWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteWindowStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteWindowStatus) -> bool {
        match self.status {
            QuoteWindowStatus::Requested => {
                match next {
                    QuoteWindowStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteWindowStatus::Granted => {
                return next == QuoteWindowStatus::Revoked;
            }
            QuoteWindowStatus::Revoked => {
                match next {
                    QuoteWindowStatus::Expired => {
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

    pub fn move_to(&mut self, next: QuoteWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ratio - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteWindow {
        QuoteWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            description: self.description.clone(),
            retries: self.retries,
            unit: self.unit.clone(),
            ratio: self.ratio,
            locale: self.locale.clone(),
            margin: self.margin,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a quote window in status requested.
pub fn new_quote_window(code: &str, title: &str) -> QuoteWindow {
    QuoteWindow {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        description: "".to_string(),
        retries: 0,
        unit: "".to_string(),
        ratio: 0.0,
        locale: "".to_string(),
        margin: 0.0,
        sequence: 0,
        status: QuoteWindowStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_quote_window("code-751", "title-548");
        assert_eq!(item.status, QuoteWindowStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_quote_window("code-751", "title-548");
        if item.move_to(QuoteWindowStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteWindowStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_quote_window("code-751", "title-548");
        item.status = QuoteWindowStatus::Expired;
        assert!(
            item.move_to(QuoteWindowStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
