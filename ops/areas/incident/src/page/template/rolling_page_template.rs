use super::*;

use acme_kit::text;

/// Where a rolling page template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPageTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingPageTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingPageTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling page template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageTemplate {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub owner_id: String,
    pub weight: f64,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub window_days: i64,
    pub status: RollingPageTemplateStatus,
}

impl RollingPageTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPageTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPageTemplateStatus) -> bool {
        match self.status {
            RollingPageTemplateStatus::Created => {
                return next == RollingPageTemplateStatus::Validated || next == RollingPageTemplateStatus::Published;
            }
            RollingPageTemplateStatus::Validated => {
                match next {
                    RollingPageTemplateStatus::Published | RollingPageTemplateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPageTemplateStatus::Published => {
                match next {
                    RollingPageTemplateStatus::Retired => {
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

    pub fn move_to(&mut self, next: RollingPageTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling page template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
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

    pub fn projected_total(&self) -> f64 {
        ((self.expires_at as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPageTemplate {
        RollingPageTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            owner_id: self.owner_id.clone(),
            weight: self.weight,
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling page template in status created.
pub fn new_rolling_page_template(code: &str, label: &str) -> RollingPageTemplate {
    RollingPageTemplate {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        owner_id: "".to_string(),
        weight: 0.0,
        expires_at: 0,
        tags: Vec::new(),
        window_days: 0,
        status: RollingPageTemplateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rolling_page_template("code-890", "label-168");
        assert_eq!(item.status, RollingPageTemplateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_rolling_page_template("code-890", "label-168");
        if item.move_to(RollingPageTemplateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPageTemplateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rolling_page_template("code-890", "label-168");
        item.status = RollingPageTemplateStatus::Retired;
        assert!(
            item.move_to(RollingPageTemplateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
