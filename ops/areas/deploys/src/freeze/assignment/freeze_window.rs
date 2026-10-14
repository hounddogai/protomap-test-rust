use super::*;

use acme_kit::text;

/// Where a freeze window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FreezeWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for FreezeWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One freeze window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeWindow {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub discount: f64,
    pub threshold: f64,
    pub retries: i64,
    pub phone: String,
    pub owner_id: String,
    pub cost: f64,
    pub issuer: String,
    pub status: FreezeWindowStatus,
}

impl FreezeWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeWindowStatus) -> bool {
        match self.status {
            FreezeWindowStatus::Open => {
                return next == FreezeWindowStatus::Archived;
            }
            FreezeWindowStatus::InReview => {
                match next {
                    FreezeWindowStatus::Reopened | FreezeWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeWindowStatus::Resolved => {
                match next {
                    FreezeWindowStatus::Reopened | FreezeWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeWindowStatus::Reopened => {
                match next {
                    FreezeWindowStatus::Archived => {
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

    pub fn move_to(&mut self, next: FreezeWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.threshold - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeWindow {
        FreezeWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            discount: self.discount,
            threshold: self.threshold,
            retries: self.retries,
            phone: self.phone.clone(),
            owner_id: self.owner_id.clone(),
            cost: self.cost,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a freeze window in status open.
pub fn new_freeze_window(code: &str, name: &str) -> FreezeWindow {
    FreezeWindow {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        discount: 0.0,
        threshold: 0.0,
        retries: 0,
        phone: "".to_string(),
        owner_id: "".to_string(),
        cost: 0.0,
        issuer: "".to_string(),
        status: FreezeWindowStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_freeze_window("code-121", "name-966");
        assert_eq!(item.status, FreezeWindowStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_freeze_window("code-121", "name-966");
        if item.move_to(FreezeWindowStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeWindowStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_freeze_window("code-121", "name-966");
        item.status = FreezeWindowStatus::Archived;
        assert!(item.move_to(FreezeWindowStatus::Open).is_err(), "expected move to fail");
    }
}
