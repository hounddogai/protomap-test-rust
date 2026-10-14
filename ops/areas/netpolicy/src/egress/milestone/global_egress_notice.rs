use super::*;

use acme_kit::text;

/// Where a global egress notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalEgressNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalEgressNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for GlobalEgressNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global egress notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressNotice {
    pub code: String,
    pub name: String,
    pub country: String,
    pub owner_id: String,
    pub expires_at: i64,
    pub weight: f64,
    pub budget: f64,
    pub channel: String,
    pub currency: String,
    pub margin: f64,
    pub contact_email: String,
    pub status: GlobalEgressNoticeStatus,
}

impl GlobalEgressNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalEgressNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalEgressNoticeStatus) -> bool {
        match self.status {
            GlobalEgressNoticeStatus::Planned => {
                return next == GlobalEgressNoticeStatus::InProgress || next == GlobalEgressNoticeStatus::Cancelled;
            }
            GlobalEgressNoticeStatus::Scheduled => {
                return next == GlobalEgressNoticeStatus::InProgress;
            }
            GlobalEgressNoticeStatus::InProgress => {
                match next {
                    GlobalEgressNoticeStatus::Completed | GlobalEgressNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEgressNoticeStatus::Completed => {
                match next {
                    GlobalEgressNoticeStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: GlobalEgressNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global egress notice {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        (self.weight * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalEgressNotice {
        GlobalEgressNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            owner_id: self.owner_id.clone(),
            expires_at: self.expires_at,
            weight: self.weight,
            budget: self.budget,
            channel: self.channel.clone(),
            currency: self.currency.clone(),
            margin: self.margin,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a global egress notice in status planned.
pub fn new_global_egress_notice(code: &str, name: &str) -> GlobalEgressNotice {
    GlobalEgressNotice {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        owner_id: "".to_string(),
        expires_at: 0,
        weight: 0.0,
        budget: 0.0,
        channel: "".to_string(),
        currency: "".to_string(),
        margin: 0.0,
        contact_email: "".to_string(),
        status: GlobalEgressNoticeStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_egress_notice("code-269", "name-448");
        assert_eq!(item.status, GlobalEgressNoticeStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_global_egress_notice("code-269", "name-448");
        if item.move_to(GlobalEgressNoticeStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalEgressNoticeStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_egress_notice("code-269", "name-448");
        item.status = GlobalEgressNoticeStatus::Cancelled;
        assert!(
            item.move_to(GlobalEgressNoticeStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
