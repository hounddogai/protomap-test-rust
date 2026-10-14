use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a health check bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckBundleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl HealthCheckBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for HealthCheckBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBundle {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub limit: i64,
    pub window_days: i64,
    pub host: String,
    pub margin: f64,
    pub discount: f64,
    pub ratio: f64,
    pub status: HealthCheckBundleStatus,
}

impl HealthCheckBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckBundleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckBundleStatus) -> bool {
        match self.status {
            HealthCheckBundleStatus::Created => {
                match next {
                    HealthCheckBundleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckBundleStatus::Validated => {
                return next == HealthCheckBundleStatus::Published;
            }
            HealthCheckBundleStatus::Published => {
                match next {
                    HealthCheckBundleStatus::Retired => {
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

    pub fn move_to(&mut self, next: HealthCheckBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "health check bundle", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ratio - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckBundle {
        HealthCheckBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            limit: self.limit,
            window_days: self.window_days,
            host: self.host.clone(),
            margin: self.margin,
            discount: self.discount,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a health check bundle in status created.
pub fn new_health_check_bundle(code: &str, title: &str) -> HealthCheckBundle {
    HealthCheckBundle {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        limit: 0,
        window_days: 0,
        host: "".to_string(),
        margin: 0.0,
        discount: 0.0,
        ratio: 0.0,
        status: HealthCheckBundleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_health_check_bundle("code-242", "title-894");
        assert_eq!(item.status, HealthCheckBundleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_health_check_bundle("code-242", "title-894");
        if item.move_to(HealthCheckBundleStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckBundleStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_health_check_bundle("code-242", "title-894");
        item.status = HealthCheckBundleStatus::Retired;
        assert!(
            item.move_to(HealthCheckBundleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
