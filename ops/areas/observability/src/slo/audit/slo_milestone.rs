use super::*;

use acme_kit::text;

/// Where a slo milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SloMilestoneStatus {
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

impl std::fmt::Display for SloMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One slo milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloMilestone {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub retries: i64,
    pub key_id: String,
    pub batch_size: i64,
    pub starts_at: i64,
    pub pinned: bool,
    pub status: SloMilestoneStatus,
}

impl SloMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloMilestoneStatus) -> bool {
        match self.status {
            SloMilestoneStatus::Draft => {
                return next == SloMilestoneStatus::Active;
            }
            SloMilestoneStatus::Pending => {
                match next {
                    SloMilestoneStatus::Active | SloMilestoneStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloMilestoneStatus::Active => {
                return next == SloMilestoneStatus::Suspended || next == SloMilestoneStatus::Closed;
            }
            SloMilestoneStatus::Suspended => {
                return next == SloMilestoneStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo milestone {} cannot move from {} to {}",
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
        format!("{} {}", "slo milestone", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.starts_at as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloMilestone {
        SloMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            retries: self.retries,
            key_id: self.key_id.clone(),
            batch_size: self.batch_size,
            starts_at: self.starts_at,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a slo milestone in status draft.
pub fn new_slo_milestone(code: &str, name: &str) -> SloMilestone {
    SloMilestone {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        retries: 0,
        key_id: "".to_string(),
        batch_size: 0,
        starts_at: 0,
        pinned: false,
        status: SloMilestoneStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_slo_milestone("code-845", "name-93");
        assert_eq!(item.status, SloMilestoneStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_slo_milestone("code-845", "name-93");
        if item.move_to(SloMilestoneStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloMilestoneStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_slo_milestone("code-845", "name-93");
        item.status = SloMilestoneStatus::Closed;
        assert!(
            item.move_to(SloMilestoneStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
