use super::*;

use acme_kit::text;

/// Where a overage target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl OverageTargetStatus {
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

impl std::fmt::Display for OverageTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageTarget {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub scope: String,
    pub duration_minutes: i64,
    pub region: String,
    pub tags: Vec<String>,
    pub starts_at: i64,
    pub unit: String,
    pub status: OverageTargetStatus,
}

impl OverageTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageTargetStatus) -> bool {
        match self.status {
            OverageTargetStatus::Draft => {
                match next {
                    OverageTargetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageTargetStatus::Pending => {
                return next == OverageTargetStatus::Closed;
            }
            OverageTargetStatus::Active => {
                match next {
                    OverageTargetStatus::Suspended | OverageTargetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageTargetStatus::Suspended => {
                match next {
                    OverageTargetStatus::Closed => {
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

    pub fn move_to(&mut self, next: OverageTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageTarget {
        OverageTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            scope: self.scope.clone(),
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            tags: self.tags.clone(),
            starts_at: self.starts_at,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a overage target in status draft.
pub fn new_overage_target(code: &str, label: &str) -> OverageTarget {
    OverageTarget {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        scope: "".to_string(),
        duration_minutes: 0,
        region: "".to_string(),
        tags: Vec::new(),
        starts_at: 0,
        unit: "".to_string(),
        status: OverageTargetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_overage_target("code-329", "label-335");
        assert_eq!(item.status, OverageTargetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_overage_target("code-329", "label-335");
        if item.move_to(OverageTargetStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageTargetStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_overage_target("code-329", "label-335");
        item.status = OverageTargetStatus::Closed;
        assert!(
            item.move_to(OverageTargetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
