use super::*;

use acme_kit::text;

/// Where a release group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseGroupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ReleaseGroupStatus {
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

impl std::fmt::Display for ReleaseGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One release group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseGroup {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub reviewer: String,
    pub unit: String,
    pub archived: bool,
    pub port: i64,
    pub ends_at: i64,
    pub discount: f64,
    pub status: ReleaseGroupStatus,
}

impl ReleaseGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseGroupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseGroupStatus) -> bool {
        match self.status {
            ReleaseGroupStatus::Draft => {
                match next {
                    ReleaseGroupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseGroupStatus::Pending => {
                return next == ReleaseGroupStatus::Active;
            }
            ReleaseGroupStatus::Active => {
                return next == ReleaseGroupStatus::Suspended;
            }
            ReleaseGroupStatus::Suspended => {
                match next {
                    ReleaseGroupStatus::Closed => {
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

    pub fn move_to(&mut self, next: ReleaseGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release group {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ends_at as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseGroup {
        ReleaseGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            reviewer: self.reviewer.clone(),
            unit: self.unit.clone(),
            archived: self.archived,
            port: self.port,
            ends_at: self.ends_at,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a release group in status draft.
pub fn new_release_group(code: &str, title: &str) -> ReleaseGroup {
    ReleaseGroup {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        reviewer: "".to_string(),
        unit: "".to_string(),
        archived: false,
        port: 0,
        ends_at: 0,
        discount: 0.0,
        status: ReleaseGroupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_release_group("code-241", "title-210");
        assert_eq!(item.status, ReleaseGroupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_release_group("code-241", "title-210");
        if item.move_to(ReleaseGroupStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseGroupStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_release_group("code-241", "title-210");
        item.status = ReleaseGroupStatus::Closed;
        assert!(
            item.move_to(ReleaseGroupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
