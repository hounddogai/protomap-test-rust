use super::*;

use acme_kit::text;

/// Where a tiered rule summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRuleSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredRuleSummaryStatus {
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

impl std::fmt::Display for TieredRuleSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered rule summary as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleSummary {
    pub code: String,
    pub title: String,
    pub label: String,
    pub channel: String,
    pub ends_at: i64,
    pub currency: String,
    pub updated_at: i64,
    pub cost: f64,
    pub contact_email: String,
    pub status: TieredRuleSummaryStatus,
}

impl TieredRuleSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRuleSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRuleSummaryStatus) -> bool {
        match self.status {
            TieredRuleSummaryStatus::Draft => {
                return next == TieredRuleSummaryStatus::Pending;
            }
            TieredRuleSummaryStatus::Pending => {
                match next {
                    TieredRuleSummaryStatus::Suspended | TieredRuleSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRuleSummaryStatus::Active => {
                match next {
                    TieredRuleSummaryStatus::Suspended | TieredRuleSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRuleSummaryStatus::Suspended => {
                return next == TieredRuleSummaryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredRuleSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered rule summary {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.updated_at as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRuleSummary {
        TieredRuleSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            ends_at: self.ends_at,
            currency: self.currency.clone(),
            updated_at: self.updated_at,
            cost: self.cost,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a tiered rule summary in status draft.
pub fn new_tiered_rule_summary(code: &str, title: &str) -> TieredRuleSummary {
    TieredRuleSummary {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        channel: "".to_string(),
        ends_at: 0,
        currency: "".to_string(),
        updated_at: 0,
        cost: 0.0,
        contact_email: "".to_string(),
        status: TieredRuleSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_rule_summary("code-346", "title-11");
        assert_eq!(item.status, TieredRuleSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_tiered_rule_summary("code-346", "title-11");
        if item.move_to(TieredRuleSummaryStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRuleSummaryStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_rule_summary("code-346", "title-11");
        item.status = TieredRuleSummaryStatus::Closed;
        assert!(
            item.move_to(TieredRuleSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
