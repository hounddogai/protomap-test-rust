use super::*;

use acme_kit::text;

/// Where a default plan entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultPlanEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultPlanEntryStatus {
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

impl std::fmt::Display for DefaultPlanEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default plan entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanEntry {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub threshold: f64,
    pub host: String,
    pub tenant_id: String,
    pub pinned: bool,
    pub checksum: String,
    pub cluster: String,
    pub status: DefaultPlanEntryStatus,
}

impl DefaultPlanEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultPlanEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultPlanEntryStatus) -> bool {
        match self.status {
            DefaultPlanEntryStatus::Draft => {
                return next == DefaultPlanEntryStatus::Pending || next == DefaultPlanEntryStatus::Active;
            }
            DefaultPlanEntryStatus::Pending => {
                match next {
                    DefaultPlanEntryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPlanEntryStatus::Active => {
                return next == DefaultPlanEntryStatus::Closed;
            }
            DefaultPlanEntryStatus::Suspended => {
                return next == DefaultPlanEntryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultPlanEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default plan entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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
        self.threshold
    }

    pub fn copy(&self) -> DefaultPlanEntry {
        DefaultPlanEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            threshold: self.threshold,
            host: self.host.clone(),
            tenant_id: self.tenant_id.clone(),
            pinned: self.pinned,
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a default plan entry in status draft.
pub fn new_default_plan_entry(code: &str, title: &str) -> DefaultPlanEntry {
    DefaultPlanEntry {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        threshold: 0.0,
        host: "".to_string(),
        tenant_id: "".to_string(),
        pinned: false,
        checksum: "".to_string(),
        cluster: "".to_string(),
        status: DefaultPlanEntryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_plan_entry("code-284", "title-871");
        assert_eq!(item.status, DefaultPlanEntryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_default_plan_entry("code-284", "title-871");
        if item.move_to(DefaultPlanEntryStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultPlanEntryStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_plan_entry("code-284", "title-871");
        item.status = DefaultPlanEntryStatus::Closed;
        assert!(
            item.move_to(DefaultPlanEntryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
