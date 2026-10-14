use super::*;

use acme_kit::text;

/// Where a policy summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicySummaryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PolicySummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PolicySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One policy summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicySummary {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub currency: String,
    pub budget: f64,
    pub batch_size: i64,
    pub port: i64,
    pub channel: String,
    pub status: PolicySummaryStatus,
}

impl PolicySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicySummaryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicySummaryStatus) -> bool {
        match self.status {
            PolicySummaryStatus::Proposed => {
                match next {
                    PolicySummaryStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicySummaryStatus::Approved => {
                match next {
                    PolicySummaryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicySummaryStatus::Rejected => {
                return next == PolicySummaryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.batch_size as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicySummary {
        PolicySummary {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            created_at: self.created_at,
            expires_at: self.expires_at,
            currency: self.currency.clone(),
            budget: self.budget,
            batch_size: self.batch_size,
            port: self.port,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a policy summary in status proposed.
pub fn new_policy_summary(code: &str, title: &str) -> PolicySummary {
    PolicySummary {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        created_at: 0,
        expires_at: 0,
        currency: "".to_string(),
        budget: 0.0,
        batch_size: 0,
        port: 0,
        channel: "".to_string(),
        status: PolicySummaryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_policy_summary("code-252", "title-590");
        assert_eq!(item.status, PolicySummaryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_policy_summary("code-252", "title-590");
        if item.move_to(PolicySummaryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicySummaryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_policy_summary("code-252", "title-590");
        item.status = PolicySummaryStatus::Withdrawn;
        assert!(
            item.move_to(PolicySummaryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
