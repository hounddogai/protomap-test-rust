use super::*;

use acme_kit::text;

/// Where a capped dashboard window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedDashboardWindowStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedDashboardWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedDashboardWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped dashboard window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardWindow {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub rotation_days: i64,
    pub namespace: String,
    pub summary: String,
    pub contact_email: String,
    pub external_ref: String,
    pub ends_at: i64,
    pub status: CappedDashboardWindowStatus,
}

impl CappedDashboardWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedDashboardWindowStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedDashboardWindowStatus) -> bool {
        match self.status {
            CappedDashboardWindowStatus::Created => {
                match next {
                    CappedDashboardWindowStatus::Published | CappedDashboardWindowStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedDashboardWindowStatus::Validated => {
                return next == CappedDashboardWindowStatus::Published || next == CappedDashboardWindowStatus::Retired;
            }
            CappedDashboardWindowStatus::Published => {
                return next == CappedDashboardWindowStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedDashboardWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped dashboard window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped dashboard window", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rotation_days as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedDashboardWindow {
        CappedDashboardWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            rotation_days: self.rotation_days,
            namespace: self.namespace.clone(),
            summary: self.summary.clone(),
            contact_email: self.contact_email.clone(),
            external_ref: self.external_ref.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a capped dashboard window in status created.
pub fn new_capped_dashboard_window(code: &str, title: &str) -> CappedDashboardWindow {
    CappedDashboardWindow {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        rotation_days: 0,
        namespace: "".to_string(),
        summary: "".to_string(),
        contact_email: "".to_string(),
        external_ref: "".to_string(),
        ends_at: 0,
        status: CappedDashboardWindowStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_dashboard_window("code-862", "title-503");
        assert_eq!(item.status, CappedDashboardWindowStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_capped_dashboard_window("code-862", "title-503");
        if item.move_to(CappedDashboardWindowStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedDashboardWindowStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_dashboard_window("code-862", "title-503");
        item.status = CappedDashboardWindowStatus::Retired;
        assert!(
            item.move_to(CappedDashboardWindowStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
