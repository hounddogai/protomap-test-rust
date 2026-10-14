use super::*;

use acme_kit::text;

/// Where a default zone summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultZoneSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultZoneSummaryStatus {
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

impl std::fmt::Display for DefaultZoneSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default zone summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneSummary {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub window_days: i64,
    pub created_at: i64,
    pub principal: String,
    pub region: String,
    pub grace_days: i64,
    pub category: String,
    pub minimum: f64,
    pub capacity: i64,
    pub status: DefaultZoneSummaryStatus,
}

impl DefaultZoneSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultZoneSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultZoneSummaryStatus) -> bool {
        match self.status {
            DefaultZoneSummaryStatus::Draft => {
                return next == DefaultZoneSummaryStatus::Closed;
            }
            DefaultZoneSummaryStatus::Pending => {
                return next == DefaultZoneSummaryStatus::Closed;
            }
            DefaultZoneSummaryStatus::Active => {
                match next {
                    DefaultZoneSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultZoneSummaryStatus::Suspended => {
                return next == DefaultZoneSummaryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultZoneSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default zone summary {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "default zone summary", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.grace_days as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultZoneSummary {
        DefaultZoneSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            window_days: self.window_days,
            created_at: self.created_at,
            principal: self.principal.clone(),
            region: self.region.clone(),
            grace_days: self.grace_days,
            category: self.category.clone(),
            minimum: self.minimum,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a default zone summary in status draft.
pub fn new_default_zone_summary(code: &str, label: &str) -> DefaultZoneSummary {
    DefaultZoneSummary {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        window_days: 0,
        created_at: 0,
        principal: "".to_string(),
        region: "".to_string(),
        grace_days: 0,
        category: "".to_string(),
        minimum: 0.0,
        capacity: 0,
        status: DefaultZoneSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_zone_summary("code-6", "label-489");
        assert_eq!(item.status, DefaultZoneSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_default_zone_summary("code-6", "label-489");
        if item.move_to(DefaultZoneSummaryStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultZoneSummaryStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_zone_summary("code-6", "label-489");
        item.status = DefaultZoneSummaryStatus::Closed;
        assert!(
            item.move_to(DefaultZoneSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
