use super::*;

use acme_kit::text;

/// Where a page preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PagePreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PagePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PagePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page preference as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PagePreference {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub starts_at: i64,
    pub updated_at: i64,
    pub floor_value: f64,
    pub locale: String,
    pub namespace: String,
    pub score: f64,
    pub status: PagePreferenceStatus,
}

impl PagePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PagePreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PagePreferenceStatus) -> bool {
        match self.status {
            PagePreferenceStatus::Created => {
                match next {
                    PagePreferenceStatus::Validated | PagePreferenceStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PagePreferenceStatus::Validated => {
                return next == PagePreferenceStatus::Published || next == PagePreferenceStatus::Retired;
            }
            PagePreferenceStatus::Published => {
                return next == PagePreferenceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PagePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page preference {} cannot move from {} to {}",
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

    pub fn projected_load(&self) -> f64 {
        ((self.score + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PagePreference {
        PagePreference {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            starts_at: self.starts_at,
            updated_at: self.updated_at,
            floor_value: self.floor_value,
            locale: self.locale.clone(),
            namespace: self.namespace.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a page preference in status created.
pub fn new_page_preference(code: &str, label: &str) -> PagePreference {
    PagePreference {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        starts_at: 0,
        updated_at: 0,
        floor_value: 0.0,
        locale: "".to_string(),
        namespace: "".to_string(),
        score: 0.0,
        status: PagePreferenceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_page_preference("code-759", "label-175");
        assert_eq!(item.status, PagePreferenceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_page_preference("code-759", "label-175");
        if item.move_to(PagePreferenceStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PagePreferenceStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_page_preference("code-759", "label-175");
        item.status = PagePreferenceStatus::Retired;
        assert!(
            item.move_to(PagePreferenceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
