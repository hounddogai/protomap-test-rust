use super::*;

use acme_kit::text;

/// Where a page milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PageMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PageMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageMilestone {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub quota_units: i64,
    pub channel: String,
    pub cluster: String,
    pub starts_at: i64,
    pub budget: f64,
    pub status: PageMilestoneStatus,
}

impl PageMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageMilestoneStatus) -> bool {
        match self.status {
            PageMilestoneStatus::Requested => {
                match next {
                    PageMilestoneStatus::Revoked | PageMilestoneStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageMilestoneStatus::Granted => {
                match next {
                    PageMilestoneStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageMilestoneStatus::Revoked => {
                match next {
                    PageMilestoneStatus::Expired => {
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

    pub fn move_to(&mut self, next: PageMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        (self.budget * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageMilestone {
        PageMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            quota_units: self.quota_units,
            channel: self.channel.clone(),
            cluster: self.cluster.clone(),
            starts_at: self.starts_at,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a page milestone in status requested.
pub fn new_page_milestone(code: &str, name: &str) -> PageMilestone {
    PageMilestone {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        quota_units: 0,
        channel: "".to_string(),
        cluster: "".to_string(),
        starts_at: 0,
        budget: 0.0,
        status: PageMilestoneStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_page_milestone("code-199", "name-223");
        assert_eq!(item.status, PageMilestoneStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_page_milestone("code-199", "name-223");
        if item.move_to(PageMilestoneStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageMilestoneStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_page_milestone("code-199", "name-223");
        item.status = PageMilestoneStatus::Expired;
        assert!(
            item.move_to(PageMilestoneStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
