use super::*;

use acme_kit::text;

/// Where a tiered overage template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredOverageTemplateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredOverageTemplateStatus {
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

impl std::fmt::Display for TieredOverageTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered overage template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageTemplate {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub description: String,
    pub principal: String,
    pub visible: bool,
    pub status: TieredOverageTemplateStatus,
}

impl TieredOverageTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredOverageTemplateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredOverageTemplateStatus) -> bool {
        match self.status {
            TieredOverageTemplateStatus::New => {
                match next {
                    TieredOverageTemplateStatus::Done | TieredOverageTemplateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredOverageTemplateStatus::Queued => {
                match next {
                    TieredOverageTemplateStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredOverageTemplateStatus::Running => {
                return next == TieredOverageTemplateStatus::Failed;
            }
            TieredOverageTemplateStatus::Done => {
                return next == TieredOverageTemplateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredOverageTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered overage template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> TieredOverageTemplate {
        TieredOverageTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            description: self.description.clone(),
            principal: self.principal.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered overage template in status new.
pub fn new_tiered_overage_template(code: &str, name: &str) -> TieredOverageTemplate {
    TieredOverageTemplate {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        description: "".to_string(),
        principal: "".to_string(),
        visible: false,
        status: TieredOverageTemplateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_overage_template("code-326", "name-857");
        assert_eq!(item.status, TieredOverageTemplateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_tiered_overage_template("code-326", "name-857");
        if item.move_to(TieredOverageTemplateStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredOverageTemplateStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_overage_template("code-326", "name-857");
        item.status = TieredOverageTemplateStatus::Failed;
        assert!(
            item.move_to(TieredOverageTemplateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
