use super::*;

use acme_kit::text;

/// Where a lenient binding preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientBindingPreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientBindingPreferenceStatus {
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

impl std::fmt::Display for LenientBindingPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient binding preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientBindingPreference {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub pinned: bool,
    pub approved: bool,
    pub status: LenientBindingPreferenceStatus,
}

impl LenientBindingPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientBindingPreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientBindingPreferenceStatus) -> bool {
        match self.status {
            LenientBindingPreferenceStatus::New => {
                match next {
                    LenientBindingPreferenceStatus::Running | LenientBindingPreferenceStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientBindingPreferenceStatus::Queued => {
                return next == LenientBindingPreferenceStatus::Running;
            }
            LenientBindingPreferenceStatus::Running => {
                match next {
                    LenientBindingPreferenceStatus::Done | LenientBindingPreferenceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientBindingPreferenceStatus::Done => {
                return next == LenientBindingPreferenceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientBindingPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient binding preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> LenientBindingPreference {
        LenientBindingPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            pinned: self.pinned,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient binding preference in status new.
pub fn new_lenient_binding_preference(code: &str, title: &str) -> LenientBindingPreference {
    LenientBindingPreference {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        pinned: false,
        approved: false,
        status: LenientBindingPreferenceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_binding_preference("code-988", "title-700");
        assert_eq!(item.status, LenientBindingPreferenceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_lenient_binding_preference("code-988", "title-700");
        if item.move_to(LenientBindingPreferenceStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientBindingPreferenceStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_binding_preference("code-988", "title-700");
        item.status = LenientBindingPreferenceStatus::Failed;
        assert!(
            item.move_to(LenientBindingPreferenceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
