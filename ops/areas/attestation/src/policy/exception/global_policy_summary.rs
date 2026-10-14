use super::*;

use acme_kit::text;

/// Where a global policy summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPolicySummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalPolicySummaryStatus {
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

impl std::fmt::Display for GlobalPolicySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global policy summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicySummary {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub currency: String,
    pub rotation_days: i64,
    pub expires_at: i64,
    pub approved: bool,
    pub status: GlobalPolicySummaryStatus,
}

impl GlobalPolicySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPolicySummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPolicySummaryStatus) -> bool {
        match self.status {
            GlobalPolicySummaryStatus::Draft => {
                return next == GlobalPolicySummaryStatus::Suspended;
            }
            GlobalPolicySummaryStatus::Pending => {
                match next {
                    GlobalPolicySummaryStatus::Active | GlobalPolicySummaryStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPolicySummaryStatus::Active => {
                return next == GlobalPolicySummaryStatus::Suspended || next == GlobalPolicySummaryStatus::Closed;
            }
            GlobalPolicySummaryStatus::Suspended => {
                match next {
                    GlobalPolicySummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalPolicySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global policy summary {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.rotation_days as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPolicySummary {
        GlobalPolicySummary {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            currency: self.currency.clone(),
            rotation_days: self.rotation_days,
            expires_at: self.expires_at,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global policy summary in status draft.
pub fn new_global_policy_summary(code: &str, label: &str) -> GlobalPolicySummary {
    GlobalPolicySummary {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        currency: "".to_string(),
        rotation_days: 0,
        expires_at: 0,
        approved: false,
        status: GlobalPolicySummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_policy_summary("code-707", "label-881");
        assert_eq!(item.status, GlobalPolicySummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_global_policy_summary("code-707", "label-881");
        if item.move_to(GlobalPolicySummaryStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPolicySummaryStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_policy_summary("code-707", "label-881");
        item.status = GlobalPolicySummaryStatus::Closed;
        assert!(
            item.move_to(GlobalPolicySummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
