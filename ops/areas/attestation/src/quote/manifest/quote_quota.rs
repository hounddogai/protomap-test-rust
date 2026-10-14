use super::*;

use acme_kit::text;

/// Where a quote quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl QuoteQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for QuoteQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One quote quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteQuota {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub weight: f64,
    pub namespace: String,
    pub timezone: String,
    pub cipher: String,
    pub expires_at: i64,
    pub external_ref: String,
    pub status: QuoteQuotaStatus,
}

impl QuoteQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteQuotaStatus) -> bool {
        match self.status {
            QuoteQuotaStatus::Created => {
                return next == QuoteQuotaStatus::Validated || next == QuoteQuotaStatus::Published;
            }
            QuoteQuotaStatus::Validated => {
                return next == QuoteQuotaStatus::Retired;
            }
            QuoteQuotaStatus::Published => {
                match next {
                    QuoteQuotaStatus::Retired => {
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

    pub fn move_to(&mut self, next: QuoteQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote quota {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.weight * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteQuota {
        QuoteQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            weight: self.weight,
            namespace: self.namespace.clone(),
            timezone: self.timezone.clone(),
            cipher: self.cipher.clone(),
            expires_at: self.expires_at,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a quote quota in status created.
pub fn new_quote_quota(code: &str, label: &str) -> QuoteQuota {
    QuoteQuota {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        weight: 0.0,
        namespace: "".to_string(),
        timezone: "".to_string(),
        cipher: "".to_string(),
        expires_at: 0,
        external_ref: "".to_string(),
        status: QuoteQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_quote_quota("code-27", "label-172");
        assert_eq!(item.status, QuoteQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_quote_quota("code-27", "label-172");
        if item.move_to(QuoteQuotaStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteQuotaStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_quote_quota("code-27", "label-172");
        item.status = QuoteQuotaStatus::Retired;
        assert!(
            item.move_to(QuoteQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
