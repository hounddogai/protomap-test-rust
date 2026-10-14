use super::*;

use acme_kit::text;

/// Where a silence plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilencePlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SilencePlanStatus {
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

impl std::fmt::Display for SilencePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One silence plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilencePlan {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub priority: i64,
    pub ttl_seconds: i64,
    pub floor_value: f64,
    pub minimum: f64,
    pub cipher: String,
    pub currency: String,
    pub status: SilencePlanStatus,
}

impl SilencePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilencePlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilencePlanStatus) -> bool {
        match self.status {
            SilencePlanStatus::Draft => {
                return next == SilencePlanStatus::Suspended;
            }
            SilencePlanStatus::Pending => {
                return next == SilencePlanStatus::Active || next == SilencePlanStatus::Closed;
            }
            SilencePlanStatus::Active => {
                match next {
                    SilencePlanStatus::Suspended | SilencePlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilencePlanStatus::Suspended => {
                match next {
                    SilencePlanStatus::Closed => {
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

    pub fn move_to(&mut self, next: SilencePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "silence plan", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilencePlan {
        SilencePlan {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            priority: self.priority,
            ttl_seconds: self.ttl_seconds,
            floor_value: self.floor_value,
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a silence plan in status draft.
pub fn new_silence_plan(code: &str, title: &str) -> SilencePlan {
    SilencePlan {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        priority: 0,
        ttl_seconds: 0,
        floor_value: 0.0,
        minimum: 0.0,
        cipher: "".to_string(),
        currency: "".to_string(),
        status: SilencePlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_silence_plan("code-89", "title-254");
        assert_eq!(item.status, SilencePlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_silence_plan("code-89", "title-254");
        if item.move_to(SilencePlanStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilencePlanStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_silence_plan("code-89", "title-254");
        item.status = SilencePlanStatus::Closed;
        assert!(item.move_to(SilencePlanStatus::Draft).is_err(), "expected move to fail");
    }
}
