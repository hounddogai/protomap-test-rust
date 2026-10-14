use super::*;

use acme_kit::text;

/// Where a freeze cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeCycleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FreezeCycleStatus {
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

impl std::fmt::Display for FreezeCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze cycle as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeCycle {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub ratio: f64,
    pub key_id: String,
    pub reviewer: String,
    pub owner_id: String,
    pub burst: i64,
    pub status: FreezeCycleStatus,
}

impl FreezeCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeCycleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeCycleStatus) -> bool {
        match self.status {
            FreezeCycleStatus::Open => {
                match next {
                    FreezeCycleStatus::InReview | FreezeCycleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeCycleStatus::InReview => {
                match next {
                    FreezeCycleStatus::Resolved | FreezeCycleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeCycleStatus::Resolved => {
                match next {
                    FreezeCycleStatus::Reopened | FreezeCycleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeCycleStatus::Reopened => {
                match next {
                    FreezeCycleStatus::Archived => {
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

    pub fn move_to(&mut self, next: FreezeCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "freeze cycle", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.burst as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeCycle {
        FreezeCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            ratio: self.ratio,
            key_id: self.key_id.clone(),
            reviewer: self.reviewer.clone(),
            owner_id: self.owner_id.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a freeze cycle in status open.
pub fn new_freeze_cycle(code: &str, name: &str) -> FreezeCycle {
    FreezeCycle {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        ratio: 0.0,
        key_id: "".to_string(),
        reviewer: "".to_string(),
        owner_id: "".to_string(),
        burst: 0,
        status: FreezeCycleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_freeze_cycle("code-288", "name-111");
        assert_eq!(item.status, FreezeCycleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_freeze_cycle("code-288", "name-111");
        if item.move_to(FreezeCycleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeCycleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_freeze_cycle("code-288", "name-111");
        item.status = FreezeCycleStatus::Archived;
        assert!(item.move_to(FreezeCycleStatus::Open).is_err(), "expected move to fail");
    }
}
