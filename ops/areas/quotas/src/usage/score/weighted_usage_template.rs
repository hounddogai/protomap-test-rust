use super::*;

use acme_kit::text;

/// Where a weighted usage template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedUsageTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedUsageTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedUsageTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted usage template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsageTemplate {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub sequence: i64,
    pub owner_id: String,
    pub minimum: f64,
    pub pinned: bool,
    pub locale: String,
    pub burst: i64,
    pub priority: i64,
    pub status: WeightedUsageTemplateStatus,
}

impl WeightedUsageTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedUsageTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedUsageTemplateStatus) -> bool {
        match self.status {
            WeightedUsageTemplateStatus::Created => {
                return next == WeightedUsageTemplateStatus::Published || next == WeightedUsageTemplateStatus::Retired;
            }
            WeightedUsageTemplateStatus::Validated => {
                return next == WeightedUsageTemplateStatus::Published || next == WeightedUsageTemplateStatus::Retired;
            }
            WeightedUsageTemplateStatus::Published => {
                return next == WeightedUsageTemplateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedUsageTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted usage template {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.burst as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedUsageTemplate {
        WeightedUsageTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            sequence: self.sequence,
            owner_id: self.owner_id.clone(),
            minimum: self.minimum,
            pinned: self.pinned,
            locale: self.locale.clone(),
            burst: self.burst,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a weighted usage template in status created.
pub fn new_weighted_usage_template(code: &str, title: &str) -> WeightedUsageTemplate {
    WeightedUsageTemplate {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        sequence: 0,
        owner_id: "".to_string(),
        minimum: 0.0,
        pinned: false,
        locale: "".to_string(),
        burst: 0,
        priority: 0,
        status: WeightedUsageTemplateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_usage_template("code-275", "title-875");
        assert_eq!(item.status, WeightedUsageTemplateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_weighted_usage_template("code-275", "title-875");
        if item.move_to(WeightedUsageTemplateStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedUsageTemplateStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_usage_template("code-275", "title-875");
        item.status = WeightedUsageTemplateStatus::Retired;
        assert!(
            item.move_to(WeightedUsageTemplateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
