use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a trust domain summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TrustDomainSummaryStatus {
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

impl std::fmt::Display for TrustDomainSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One trust domain summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSummary {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub attempts: i64,
    pub weight: f64,
    pub minimum: f64,
    pub label: String,
    pub ends_at: i64,
    pub maximum: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: TrustDomainSummaryStatus,
}

impl TrustDomainSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainSummaryStatus) -> bool {
        match self.status {
            TrustDomainSummaryStatus::Draft => {
                return next == TrustDomainSummaryStatus::Pending;
            }
            TrustDomainSummaryStatus::Pending => {
                match next {
                    TrustDomainSummaryStatus::Active | TrustDomainSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainSummaryStatus::Active => {
                return next == TrustDomainSummaryStatus::Closed;
            }
            TrustDomainSummaryStatus::Suspended => {
                match next {
                    TrustDomainSummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: TrustDomainSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain summary {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.minimum - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainSummary {
        TrustDomainSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            attempts: self.attempts,
            weight: self.weight,
            minimum: self.minimum,
            label: self.label.clone(),
            ends_at: self.ends_at,
            maximum: self.maximum,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a trust domain summary in status draft.
pub fn new_trust_domain_summary(code: &str, title: &str) -> TrustDomainSummary {
    TrustDomainSummary {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        attempts: 0,
        weight: 0.0,
        minimum: 0.0,
        label: "".to_string(),
        ends_at: 0,
        maximum: 0.0,
        attributes: BTreeMap::new(),
        status: TrustDomainSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_trust_domain_summary("code-485", "title-997");
        assert_eq!(item.status, TrustDomainSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_trust_domain_summary("code-485", "title-997");
        if item.move_to(TrustDomainSummaryStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainSummaryStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_trust_domain_summary("code-485", "title-997");
        item.status = TrustDomainSummaryStatus::Closed;
        assert!(
            item.move_to(TrustDomainSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
