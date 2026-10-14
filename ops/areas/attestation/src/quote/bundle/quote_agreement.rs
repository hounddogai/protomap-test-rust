use super::*;

use acme_kit::text;

/// Where a quote agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl QuoteAgreementStatus {
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

impl std::fmt::Display for QuoteAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAgreement {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub burst: i64,
    pub amount: f64,
    pub notes: String,
    pub active: bool,
    pub updated_at: i64,
    pub rotation_days: i64,
    pub revision: i64,
    pub checksum: String,
    pub status: QuoteAgreementStatus,
}

impl QuoteAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteAgreementStatus) -> bool {
        match self.status {
            QuoteAgreementStatus::Draft => {
                match next {
                    QuoteAgreementStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteAgreementStatus::Pending => {
                return next == QuoteAgreementStatus::Active || next == QuoteAgreementStatus::Suspended;
            }
            QuoteAgreementStatus::Active => {
                return next == QuoteAgreementStatus::Suspended || next == QuoteAgreementStatus::Closed;
            }
            QuoteAgreementStatus::Suspended => {
                match next {
                    QuoteAgreementStatus::Closed => {
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

    pub fn move_to(&mut self, next: QuoteAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "quote agreement", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.burst as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteAgreement {
        QuoteAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            burst: self.burst,
            amount: self.amount,
            notes: self.notes.clone(),
            active: self.active,
            updated_at: self.updated_at,
            rotation_days: self.rotation_days,
            revision: self.revision,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a quote agreement in status draft.
pub fn new_quote_agreement(code: &str, title: &str) -> QuoteAgreement {
    QuoteAgreement {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        burst: 0,
        amount: 0.0,
        notes: "".to_string(),
        active: false,
        updated_at: 0,
        rotation_days: 0,
        revision: 0,
        checksum: "".to_string(),
        status: QuoteAgreementStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_quote_agreement("code-147", "title-275");
        assert_eq!(item.status, QuoteAgreementStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_quote_agreement("code-147", "title-275");
        if item.move_to(QuoteAgreementStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteAgreementStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_quote_agreement("code-147", "title-275");
        item.status = QuoteAgreementStatus::Closed;
        assert!(
            item.move_to(QuoteAgreementStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
