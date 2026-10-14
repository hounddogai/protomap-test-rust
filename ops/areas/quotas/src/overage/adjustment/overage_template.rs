use super::*;

use acme_kit::text;

/// Where a overage template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageTemplateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl OverageTemplateStatus {
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

impl std::fmt::Display for OverageTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage template as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageTemplate {
    pub code: String,
    pub name: String,
    pub region: String,
    pub tenant_id: String,
    pub maximum: f64,
    pub source: String,
    pub tags: Vec<String>,
    pub active: bool,
    pub priority: i64,
    pub updated_at: i64,
    pub status: OverageTemplateStatus,
}

impl OverageTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageTemplateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageTemplateStatus) -> bool {
        match self.status {
            OverageTemplateStatus::Draft => {
                match next {
                    OverageTemplateStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageTemplateStatus::Pending => {
                return next == OverageTemplateStatus::Active;
            }
            OverageTemplateStatus::Active => {
                match next {
                    OverageTemplateStatus::Suspended | OverageTemplateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageTemplateStatus::Suspended => {
                match next {
                    OverageTemplateStatus::Closed => {
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

    pub fn move_to(&mut self, next: OverageTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage template {} cannot move from {} to {}",
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

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_value(&self) -> f64 {
        ((self.priority as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageTemplate {
        OverageTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            tenant_id: self.tenant_id.clone(),
            maximum: self.maximum,
            source: self.source.clone(),
            tags: self.tags.clone(),
            active: self.active,
            priority: self.priority,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a overage template in status draft.
pub fn new_overage_template(code: &str, name: &str) -> OverageTemplate {
    OverageTemplate {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        tenant_id: "".to_string(),
        maximum: 0.0,
        source: "".to_string(),
        tags: Vec::new(),
        active: false,
        priority: 0,
        updated_at: 0,
        status: OverageTemplateStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_overage_template("code-95", "name-352");
        assert_eq!(item.status, OverageTemplateStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_overage_template("code-95", "name-352");
        if item.move_to(OverageTemplateStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageTemplateStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_overage_template("code-95", "name-352");
        item.status = OverageTemplateStatus::Closed;
        assert!(
            item.move_to(OverageTemplateStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
