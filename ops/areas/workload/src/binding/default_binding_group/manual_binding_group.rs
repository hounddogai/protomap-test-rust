use super::*;

use acme_kit::text;

/// Where a manual binding group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualBindingGroupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualBindingGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualBindingGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual binding group as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingGroup {
    pub code: String,
    pub name: String,
    pub title: String,
    pub maximum: f64,
    pub threshold: f64,
    pub status: ManualBindingGroupStatus,
}

impl ManualBindingGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualBindingGroupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualBindingGroupStatus) -> bool {
        match self.status {
            ManualBindingGroupStatus::Created => {
                match next {
                    ManualBindingGroupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualBindingGroupStatus::Validated => {
                match next {
                    ManualBindingGroupStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualBindingGroupStatus::Published => {
                return next == ManualBindingGroupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualBindingGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual binding group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual binding group", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.threshold + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualBindingGroup {
        ManualBindingGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual binding group in status created.
pub fn new_manual_binding_group(code: &str, name: &str) -> ManualBindingGroup {
    ManualBindingGroup {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        maximum: 0.0,
        threshold: 0.0,
        status: ManualBindingGroupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_binding_group("code-477", "name-247");
        assert_eq!(item.status, ManualBindingGroupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_manual_binding_group("code-477", "name-247");
        if item.move_to(ManualBindingGroupStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualBindingGroupStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_binding_group("code-477", "name-247");
        item.status = ManualBindingGroupStatus::Retired;
        assert!(
            item.move_to(ManualBindingGroupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
