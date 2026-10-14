use super::*;

use acme_kit::text;

/// Where a tiered binding group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredBindingGroupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredBindingGroupStatus {
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

impl std::fmt::Display for TieredBindingGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered binding group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingGroup {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub region: String,
    pub fingerprint: String,
    pub country: String,
    pub tenant_id: String,
    pub phone: String,
    pub sequence: i64,
    pub checksum: String,
    pub status: TieredBindingGroupStatus,
}

impl TieredBindingGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredBindingGroupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredBindingGroupStatus) -> bool {
        match self.status {
            TieredBindingGroupStatus::New => {
                match next {
                    TieredBindingGroupStatus::Running | TieredBindingGroupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredBindingGroupStatus::Queued => {
                return next == TieredBindingGroupStatus::Running || next == TieredBindingGroupStatus::Done;
            }
            TieredBindingGroupStatus::Running => {
                return next == TieredBindingGroupStatus::Done || next == TieredBindingGroupStatus::Failed;
            }
            TieredBindingGroupStatus::Done => {
                return next == TieredBindingGroupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredBindingGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered binding group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        (self.ratio * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredBindingGroup {
        TieredBindingGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            region: self.region.clone(),
            fingerprint: self.fingerprint.clone(),
            country: self.country.clone(),
            tenant_id: self.tenant_id.clone(),
            phone: self.phone.clone(),
            sequence: self.sequence,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tiered binding group in status new.
pub fn new_tiered_binding_group(code: &str, title: &str) -> TieredBindingGroup {
    TieredBindingGroup {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        region: "".to_string(),
        fingerprint: "".to_string(),
        country: "".to_string(),
        tenant_id: "".to_string(),
        phone: "".to_string(),
        sequence: 0,
        checksum: "".to_string(),
        status: TieredBindingGroupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_binding_group("code-159", "title-706");
        assert_eq!(item.status, TieredBindingGroupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_tiered_binding_group("code-159", "title-706");
        if item.move_to(TieredBindingGroupStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredBindingGroupStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_binding_group("code-159", "title-706");
        item.status = TieredBindingGroupStatus::Failed;
        assert!(
            item.move_to(TieredBindingGroupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
