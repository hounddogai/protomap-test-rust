use super::*;

use acme_kit::text;

/// Where a limit preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitPreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LimitPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LimitPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit preference as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitPreference {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub unit: String,
    pub floor_value: f64,
    pub scope: String,
    pub pinned: bool,
    pub status: LimitPreferenceStatus,
}

impl LimitPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitPreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitPreferenceStatus) -> bool {
        match self.status {
            LimitPreferenceStatus::Created => {
                match next {
                    LimitPreferenceStatus::Published | LimitPreferenceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitPreferenceStatus::Validated => {
                match next {
                    LimitPreferenceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitPreferenceStatus::Published => {
                return next == LimitPreferenceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.quantity as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitPreference {
        LimitPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            unit: self.unit.clone(),
            floor_value: self.floor_value,
            scope: self.scope.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a limit preference in status created.
pub fn new_limit_preference(code: &str, title: &str) -> LimitPreference {
    LimitPreference {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        unit: "".to_string(),
        floor_value: 0.0,
        scope: "".to_string(),
        pinned: false,
        status: LimitPreferenceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_limit_preference("code-185", "title-334");
        assert_eq!(item.status, LimitPreferenceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_limit_preference("code-185", "title-334");
        if item.move_to(LimitPreferenceStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitPreferenceStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_limit_preference("code-185", "title-334");
        item.status = LimitPreferenceStatus::Retired;
        assert!(
            item.move_to(LimitPreferenceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
