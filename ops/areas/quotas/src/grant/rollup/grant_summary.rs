use super::*;

use acme_kit::text;

/// Where a grant summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GrantSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GrantSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a grant summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantSummary {
    pub code: String,
    pub label: String,
    pub category: String,
    pub unit: String,
    pub timezone: String,
    pub issuer: String,
    pub namespace: String,
    pub threshold: f64,
    pub status: GrantSummaryStatus,
}

impl GrantSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantSummaryStatus) -> bool {
        match self.status {
            GrantSummaryStatus::Requested => {
                return next == GrantSummaryStatus::Expired;
            }
            GrantSummaryStatus::Granted => {
                match next {
                    GrantSummaryStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantSummaryStatus::Revoked => {
                match next {
                    GrantSummaryStatus::Expired => {
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

    pub fn move_to(&mut self, next: GrantSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "grant summary", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> GrantSummary {
        GrantSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            unit: self.unit.clone(),
            timezone: self.timezone.clone(),
            issuer: self.issuer.clone(),
            namespace: self.namespace.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a grant summary in status requested.
pub fn new_grant_summary(code: &str, label: &str) -> GrantSummary {
    GrantSummary {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        unit: "".to_string(),
        timezone: "".to_string(),
        issuer: "".to_string(),
        namespace: "".to_string(),
        threshold: 0.0,
        status: GrantSummaryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_grant_summary("code-962", "label-254");
        assert_eq!(item.status, GrantSummaryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_grant_summary("code-962", "label-254");
        if item.move_to(GrantSummaryStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantSummaryStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_grant_summary("code-962", "label-254");
        item.status = GrantSummaryStatus::Expired;
        assert!(
            item.move_to(GrantSummaryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
