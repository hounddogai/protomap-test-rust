use super::*;

use acme_kit::text;

/// Where a adaptive isolation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveIsolationTemplateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveIsolationTemplateStatus {
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

impl std::fmt::Display for AdaptiveIsolationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive isolation template as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIsolationTemplate {
    pub code: String,
    pub label: String,
    pub region: String,
    pub approved: bool,
    pub external_ref: String,
    pub duration_minutes: i64,
    pub principal: String,
    pub owner_id: String,
    pub rate: f64,
    pub visible: bool,
    pub status: AdaptiveIsolationTemplateStatus,
}

impl AdaptiveIsolationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveIsolationTemplateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveIsolationTemplateStatus) -> bool {
        match self.status {
            AdaptiveIsolationTemplateStatus::Draft => {
                return next == AdaptiveIsolationTemplateStatus::Closed;
            }
            AdaptiveIsolationTemplateStatus::Pending => {
                return next == AdaptiveIsolationTemplateStatus::Active;
            }
            AdaptiveIsolationTemplateStatus::Active => {
                match next {
                    AdaptiveIsolationTemplateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveIsolationTemplateStatus::Suspended => {
                match next {
                    AdaptiveIsolationTemplateStatus::Closed => {
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

    pub fn move_to(&mut self, next: AdaptiveIsolationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive isolation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive isolation template", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.rate - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveIsolationTemplate {
        AdaptiveIsolationTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            approved: self.approved,
            external_ref: self.external_ref.clone(),
            duration_minutes: self.duration_minutes,
            principal: self.principal.clone(),
            owner_id: self.owner_id.clone(),
            rate: self.rate,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a adaptive isolation template in status draft.
pub fn new_adaptive_isolation_template(code: &str, label: &str) -> AdaptiveIsolationTemplate {
    AdaptiveIsolationTemplate {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        approved: false,
        external_ref: "".to_string(),
        duration_minutes: 0,
        principal: "".to_string(),
        owner_id: "".to_string(),
        rate: 0.0,
        visible: false,
        status: AdaptiveIsolationTemplateStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_isolation_template("code-787", "label-941");
        assert_eq!(item.status, AdaptiveIsolationTemplateStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_adaptive_isolation_template("code-787", "label-941");
        if item.move_to(AdaptiveIsolationTemplateStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveIsolationTemplateStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_isolation_template("code-787", "label-941");
        item.status = AdaptiveIsolationTemplateStatus::Closed;
        assert!(
            item.move_to(AdaptiveIsolationTemplateStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
