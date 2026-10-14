use super::*;

use acme_kit::text;

/// Where a adaptive grant template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveGrantTemplateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveGrantTemplateStatus {
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

impl std::fmt::Display for AdaptiveGrantTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive grant template as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveGrantTemplate {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub reviewer: String,
    pub source: String,
    pub visible: bool,
    pub ends_at: i64,
    pub version: i64,
    pub rotation_days: i64,
    pub status: AdaptiveGrantTemplateStatus,
}

impl AdaptiveGrantTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveGrantTemplateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveGrantTemplateStatus) -> bool {
        match self.status {
            AdaptiveGrantTemplateStatus::New => {
                match next {
                    AdaptiveGrantTemplateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveGrantTemplateStatus::Queued => {
                match next {
                    AdaptiveGrantTemplateStatus::Running | AdaptiveGrantTemplateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveGrantTemplateStatus::Running => {
                match next {
                    AdaptiveGrantTemplateStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveGrantTemplateStatus::Done => {
                return next == AdaptiveGrantTemplateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveGrantTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive grant template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.discount + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveGrantTemplate {
        AdaptiveGrantTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            reviewer: self.reviewer.clone(),
            source: self.source.clone(),
            visible: self.visible,
            ends_at: self.ends_at,
            version: self.version,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive grant template in status new.
pub fn new_adaptive_grant_template(code: &str, title: &str) -> AdaptiveGrantTemplate {
    AdaptiveGrantTemplate {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        reviewer: "".to_string(),
        source: "".to_string(),
        visible: false,
        ends_at: 0,
        version: 0,
        rotation_days: 0,
        status: AdaptiveGrantTemplateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_grant_template("code-16", "title-636");
        assert_eq!(item.status, AdaptiveGrantTemplateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_adaptive_grant_template("code-16", "title-636");
        if item.move_to(AdaptiveGrantTemplateStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveGrantTemplateStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_grant_template("code-16", "title-636");
        item.status = AdaptiveGrantTemplateStatus::Failed;
        assert!(
            item.move_to(AdaptiveGrantTemplateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
