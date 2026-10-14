use super::*;

use acme_kit::text;

/// Where a legacy allowlist baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAllowlistBaselineStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyAllowlistBaselineStatus {
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

impl std::fmt::Display for LegacyAllowlistBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy allowlist baseline as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBaseline {
    pub code: String,
    pub label: String,
    pub title: String,
    pub locale: String,
    pub ratio: f64,
    pub visible: bool,
    pub tags: Vec<String>,
    pub batch_size: i64,
    pub status: LegacyAllowlistBaselineStatus,
}

impl LegacyAllowlistBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAllowlistBaselineStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAllowlistBaselineStatus) -> bool {
        match self.status {
            LegacyAllowlistBaselineStatus::New => {
                return next == LegacyAllowlistBaselineStatus::Running || next == LegacyAllowlistBaselineStatus::Failed;
            }
            LegacyAllowlistBaselineStatus::Queued => {
                return next == LegacyAllowlistBaselineStatus::Running || next == LegacyAllowlistBaselineStatus::Failed;
            }
            LegacyAllowlistBaselineStatus::Running => {
                match next {
                    LegacyAllowlistBaselineStatus::Done | LegacyAllowlistBaselineStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistBaselineStatus::Done => {
                match next {
                    LegacyAllowlistBaselineStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyAllowlistBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy allowlist baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_total(&self) -> f64 {
        ((self.ratio - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyAllowlistBaseline {
        LegacyAllowlistBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            ratio: self.ratio,
            visible: self.visible,
            tags: self.tags.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a legacy allowlist baseline in status new.
pub fn new_legacy_allowlist_baseline(code: &str, label: &str) -> LegacyAllowlistBaseline {
    LegacyAllowlistBaseline {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        locale: "".to_string(),
        ratio: 0.0,
        visible: false,
        tags: Vec::new(),
        batch_size: 0,
        status: LegacyAllowlistBaselineStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_allowlist_baseline("code-813", "label-222");
        assert_eq!(item.status, LegacyAllowlistBaselineStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_legacy_allowlist_baseline("code-813", "label-222");
        if item.move_to(LegacyAllowlistBaselineStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyAllowlistBaselineStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_allowlist_baseline("code-813", "label-222");
        item.status = LegacyAllowlistBaselineStatus::Failed;
        assert!(
            item.move_to(LegacyAllowlistBaselineStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
