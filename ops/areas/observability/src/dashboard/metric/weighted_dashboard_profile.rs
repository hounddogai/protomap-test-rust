use super::*;

use acme_kit::text;

/// Where a weighted dashboard profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedDashboardProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedDashboardProfileStatus {
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

impl std::fmt::Display for WeightedDashboardProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted dashboard profile as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDashboardProfile {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub channel: String,
    pub sequence: i64,
    pub approved: bool,
    pub rate: f64,
    pub contact_email: String,
    pub status: WeightedDashboardProfileStatus,
}

impl WeightedDashboardProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedDashboardProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedDashboardProfileStatus) -> bool {
        match self.status {
            WeightedDashboardProfileStatus::Draft => {
                match next {
                    WeightedDashboardProfileStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedDashboardProfileStatus::Pending => {
                return next == WeightedDashboardProfileStatus::Closed;
            }
            WeightedDashboardProfileStatus::Active => {
                return next == WeightedDashboardProfileStatus::Suspended;
            }
            WeightedDashboardProfileStatus::Suspended => {
                match next {
                    WeightedDashboardProfileStatus::Closed => {
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

    pub fn move_to(&mut self, next: WeightedDashboardProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted dashboard profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.rate + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedDashboardProfile {
        WeightedDashboardProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            channel: self.channel.clone(),
            sequence: self.sequence,
            approved: self.approved,
            rate: self.rate,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a weighted dashboard profile in status draft.
pub fn new_weighted_dashboard_profile(code: &str, title: &str) -> WeightedDashboardProfile {
    WeightedDashboardProfile {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        channel: "".to_string(),
        sequence: 0,
        approved: false,
        rate: 0.0,
        contact_email: "".to_string(),
        status: WeightedDashboardProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_weighted_dashboard_profile("code-662", "title-25");
        assert_eq!(item.status, WeightedDashboardProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_weighted_dashboard_profile("code-662", "title-25");
        if item.move_to(WeightedDashboardProfileStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedDashboardProfileStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_weighted_dashboard_profile("code-662", "title-25");
        item.status = WeightedDashboardProfileStatus::Closed;
        assert!(
            item.move_to(WeightedDashboardProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
