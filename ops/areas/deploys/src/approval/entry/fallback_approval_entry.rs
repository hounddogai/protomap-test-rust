use super::*;

use acme_kit::text;

/// Where a fallback approval entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackApprovalEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackApprovalEntryStatus {
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

impl std::fmt::Display for FallbackApprovalEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback approval entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalEntry {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub rate: f64,
    pub principal: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub tenant_id: String,
    pub locale: String,
    pub region: String,
    pub threshold: f64,
    pub status: FallbackApprovalEntryStatus,
}

impl FallbackApprovalEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackApprovalEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackApprovalEntryStatus) -> bool {
        match self.status {
            FallbackApprovalEntryStatus::Draft => {
                return next == FallbackApprovalEntryStatus::Suspended;
            }
            FallbackApprovalEntryStatus::Pending => {
                return next == FallbackApprovalEntryStatus::Suspended || next == FallbackApprovalEntryStatus::Closed;
            }
            FallbackApprovalEntryStatus::Active => {
                return next == FallbackApprovalEntryStatus::Suspended;
            }
            FallbackApprovalEntryStatus::Suspended => {
                match next {
                    FallbackApprovalEntryStatus::Closed => {
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

    pub fn move_to(&mut self, next: FallbackApprovalEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback approval entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback approval entry", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        (self.amount * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackApprovalEntry {
        FallbackApprovalEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            rate: self.rate,
            principal: self.principal.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            tenant_id: self.tenant_id.clone(),
            locale: self.locale.clone(),
            region: self.region.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback approval entry in status draft.
pub fn new_fallback_approval_entry(code: &str, label: &str) -> FallbackApprovalEntry {
    FallbackApprovalEntry {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        rate: 0.0,
        principal: "".to_string(),
        summary: "".to_string(),
        tags: Vec::new(),
        tenant_id: "".to_string(),
        locale: "".to_string(),
        region: "".to_string(),
        threshold: 0.0,
        status: FallbackApprovalEntryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_approval_entry("code-586", "label-852");
        assert_eq!(item.status, FallbackApprovalEntryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_fallback_approval_entry("code-586", "label-852");
        if item.move_to(FallbackApprovalEntryStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackApprovalEntryStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_approval_entry("code-586", "label-852");
        item.status = FallbackApprovalEntryStatus::Closed;
        assert!(
            item.move_to(FallbackApprovalEntryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
