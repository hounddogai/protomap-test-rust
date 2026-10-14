use super::*;

use acme_kit::text;

/// Where a capped data export policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedDataExportPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedDataExportPolicyStatus {
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

impl std::fmt::Display for CappedDataExportPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped data export policy as the data export team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportPolicy {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub category: String,
    pub tags: Vec<String>,
    pub quantity: i64,
    pub status: CappedDataExportPolicyStatus,
}

impl CappedDataExportPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedDataExportPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedDataExportPolicyStatus) -> bool {
        match self.status {
            CappedDataExportPolicyStatus::New => {
                match next {
                    CappedDataExportPolicyStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDataExportPolicyStatus::Queued => {
                return next == CappedDataExportPolicyStatus::Running || next == CappedDataExportPolicyStatus::Failed;
            }
            CappedDataExportPolicyStatus::Running => {
                match next {
                    CappedDataExportPolicyStatus::Done | CappedDataExportPolicyStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDataExportPolicyStatus::Done => {
                return next == CappedDataExportPolicyStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedDataExportPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped data export policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> CappedDataExportPolicy {
        CappedDataExportPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            category: self.category.clone(),
            tags: self.tags.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped data export policy in status new.
pub fn new_capped_data_export_policy(code: &str, title: &str) -> CappedDataExportPolicy {
    CappedDataExportPolicy {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        category: "".to_string(),
        tags: Vec::new(),
        quantity: 0,
        status: CappedDataExportPolicyStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_data_export_policy("code-231", "title-235");
        assert_eq!(item.status, CappedDataExportPolicyStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_capped_data_export_policy("code-231", "title-235");
        if item.move_to(CappedDataExportPolicyStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedDataExportPolicyStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_data_export_policy("code-231", "title-235");
        item.status = CappedDataExportPolicyStatus::Failed;
        assert!(
            item.move_to(CappedDataExportPolicyStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
