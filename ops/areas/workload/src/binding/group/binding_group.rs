use super::*;

use acme_kit::text;

/// Where a binding group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingGroupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BindingGroupStatus {
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

impl std::fmt::Display for BindingGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingGroup {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub batch_size: i64,
    pub revision: i64,
    pub cipher: String,
    pub notes: String,
    pub starts_at: i64,
    pub owner_id: String,
    pub reviewer: String,
    pub status: BindingGroupStatus,
}

impl BindingGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingGroupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingGroupStatus) -> bool {
        match self.status {
            BindingGroupStatus::Draft => {
                return next == BindingGroupStatus::Suspended;
            }
            BindingGroupStatus::Pending => {
                return next == BindingGroupStatus::Active;
            }
            BindingGroupStatus::Active => {
                match next {
                    BindingGroupStatus::Suspended | BindingGroupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingGroupStatus::Suspended => {
                match next {
                    BindingGroupStatus::Closed => {
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

    pub fn move_to(&mut self, next: BindingGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.starts_at as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingGroup {
        BindingGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            batch_size: self.batch_size,
            revision: self.revision,
            cipher: self.cipher.clone(),
            notes: self.notes.clone(),
            starts_at: self.starts_at,
            owner_id: self.owner_id.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a binding group in status draft.
pub fn new_binding_group(code: &str, label: &str) -> BindingGroup {
    BindingGroup {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        batch_size: 0,
        revision: 0,
        cipher: "".to_string(),
        notes: "".to_string(),
        starts_at: 0,
        owner_id: "".to_string(),
        reviewer: "".to_string(),
        status: BindingGroupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_binding_group("code-407", "label-424");
        assert_eq!(item.status, BindingGroupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_binding_group("code-407", "label-424");
        if item.move_to(BindingGroupStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingGroupStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_binding_group("code-407", "label-424");
        item.status = BindingGroupStatus::Closed;
        assert!(
            item.move_to(BindingGroupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
