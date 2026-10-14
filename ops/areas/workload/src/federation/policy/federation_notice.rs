use super::*;

use acme_kit::text;

/// Where a federation notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FederationNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for FederationNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation notice as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationNotice {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub limit: i64,
    pub duration_minutes: i64,
    pub phone: String,
    pub status: FederationNoticeStatus,
}

impl FederationNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationNoticeStatus) -> bool {
        match self.status {
            FederationNoticeStatus::New => {
                match next {
                    FederationNoticeStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationNoticeStatus::Queued => {
                match next {
                    FederationNoticeStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationNoticeStatus::Running => {
                return next == FederationNoticeStatus::Done || next == FederationNoticeStatus::Failed;
            }
            FederationNoticeStatus::Done => {
                match next {
                    FederationNoticeStatus::Failed => {
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

    pub fn move_to(&mut self, next: FederationNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.duration_minutes as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationNotice {
        FederationNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            limit: self.limit,
            duration_minutes: self.duration_minutes,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a federation notice in status new.
pub fn new_federation_notice(code: &str, title: &str) -> FederationNotice {
    FederationNotice {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        limit: 0,
        duration_minutes: 0,
        phone: "".to_string(),
        status: FederationNoticeStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_federation_notice("code-8", "title-258");
        assert_eq!(item.status, FederationNoticeStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_federation_notice("code-8", "title-258");
        if item.move_to(FederationNoticeStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationNoticeStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_federation_notice("code-8", "title-258");
        item.status = FederationNoticeStatus::Failed;
        assert!(
            item.move_to(FederationNoticeStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
