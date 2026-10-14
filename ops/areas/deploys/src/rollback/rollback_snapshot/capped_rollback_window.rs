use super::*;

use acme_kit::text;

/// Where a capped rollback window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRollbackWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedRollbackWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedRollbackWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped rollback window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackWindow {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub category: String,
    pub window_days: i64,
    pub timezone: String,
    pub tags: Vec<String>,
    pub status: CappedRollbackWindowStatus,
}

impl CappedRollbackWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRollbackWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRollbackWindowStatus) -> bool {
        match self.status {
            CappedRollbackWindowStatus::Created => {
                match next {
                    CappedRollbackWindowStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRollbackWindowStatus::Validated => {
                return next == CappedRollbackWindowStatus::Published || next == CappedRollbackWindowStatus::Retired;
            }
            CappedRollbackWindowStatus::Published => {
                return next == CappedRollbackWindowStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRollbackWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped rollback window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.quantity as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRollbackWindow {
        CappedRollbackWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            category: self.category.clone(),
            window_days: self.window_days,
            timezone: self.timezone.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped rollback window in status created.
pub fn new_capped_rollback_window(code: &str, title: &str) -> CappedRollbackWindow {
    CappedRollbackWindow {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        category: "".to_string(),
        window_days: 0,
        timezone: "".to_string(),
        tags: Vec::new(),
        status: CappedRollbackWindowStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_rollback_window("code-437", "title-30");
        assert_eq!(item.status, CappedRollbackWindowStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_capped_rollback_window("code-437", "title-30");
        if item.move_to(CappedRollbackWindowStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedRollbackWindowStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_rollback_window("code-437", "title-30");
        item.status = CappedRollbackWindowStatus::Retired;
        assert!(
            item.move_to(CappedRollbackWindowStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
