use super::*;

use acme_kit::text;

/// Where a redaction metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RedactionMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RedactionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction metric as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMetric {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub label: String,
    pub notes: String,
    pub name: String,
    pub unit: String,
    pub version: i64,
    pub created_at: i64,
    pub status: RedactionMetricStatus,
}

impl RedactionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionMetricStatus) -> bool {
        match self.status {
            RedactionMetricStatus::Created => {
                return next == RedactionMetricStatus::Published;
            }
            RedactionMetricStatus::Validated => {
                match next {
                    RedactionMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionMetricStatus::Published => {
                match next {
                    RedactionMetricStatus::Retired => {
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

    pub fn move_to(&mut self, next: RedactionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.created_at as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionMetric {
        RedactionMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            version: self.version,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a redaction metric in status created.
pub fn new_redaction_metric(code: &str, title: &str) -> RedactionMetric {
    RedactionMetric {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        label: "".to_string(),
        notes: "".to_string(),
        name: "".to_string(),
        unit: "".to_string(),
        version: 0,
        created_at: 0,
        status: RedactionMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_redaction_metric("code-410", "title-827");
        assert_eq!(item.status, RedactionMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_redaction_metric("code-410", "title-827");
        if item.move_to(RedactionMetricStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionMetricStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_redaction_metric("code-410", "title-827");
        item.status = RedactionMetricStatus::Retired;
        assert!(
            item.move_to(RedactionMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
