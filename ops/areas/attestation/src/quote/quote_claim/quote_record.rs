use super::*;

use acme_kit::text;

/// Where a quote record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl QuoteRecordStatus {
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

impl std::fmt::Display for QuoteRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote record and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteRecord {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub visible: bool,
    pub segment: String,
    pub key_id: String,
    pub external_ref: String,
    pub budget: f64,
    pub fingerprint: String,
    pub updated_at: i64,
    pub status: QuoteRecordStatus,
}

impl QuoteRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteRecordStatus) -> bool {
        match self.status {
            QuoteRecordStatus::Draft => {
                match next {
                    QuoteRecordStatus::Suspended | QuoteRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteRecordStatus::Pending => {
                match next {
                    QuoteRecordStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteRecordStatus::Active => {
                return next == QuoteRecordStatus::Suspended || next == QuoteRecordStatus::Closed;
            }
            QuoteRecordStatus::Suspended => {
                match next {
                    QuoteRecordStatus::Closed => {
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

    pub fn move_to(&mut self, next: QuoteRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote record {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.updated_at as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteRecord {
        QuoteRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            visible: self.visible,
            segment: self.segment.clone(),
            key_id: self.key_id.clone(),
            external_ref: self.external_ref.clone(),
            budget: self.budget,
            fingerprint: self.fingerprint.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a quote record in status draft.
pub fn new_quote_record(code: &str, label: &str) -> QuoteRecord {
    QuoteRecord {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        visible: false,
        segment: "".to_string(),
        key_id: "".to_string(),
        external_ref: "".to_string(),
        budget: 0.0,
        fingerprint: "".to_string(),
        updated_at: 0,
        status: QuoteRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_quote_record("code-541", "label-225");
        assert_eq!(item.status, QuoteRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_quote_record("code-541", "label-225");
        if item.move_to(QuoteRecordStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteRecordStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_quote_record("code-541", "label-225");
        item.status = QuoteRecordStatus::Closed;
        assert!(item.move_to(QuoteRecordStatus::Draft).is_err(), "expected move to fail");
    }
}
