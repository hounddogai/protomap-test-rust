use super::*;

use acme_kit::text;

/// Where a flow log template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FlowLogTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FlowLogTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log template as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogTemplate {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub timezone: String,
    pub source: String,
    pub status: FlowLogTemplateStatus,
}

impl FlowLogTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogTemplateStatus) -> bool {
        match self.status {
            FlowLogTemplateStatus::Created => {
                match next {
                    FlowLogTemplateStatus::Validated | FlowLogTemplateStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogTemplateStatus::Validated => {
                match next {
                    FlowLogTemplateStatus::Published | FlowLogTemplateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogTemplateStatus::Published => {
                return next == FlowLogTemplateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "flow log template", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> FlowLogTemplate {
        FlowLogTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            timezone: self.timezone.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a flow log template in status created.
pub fn new_flow_log_template(code: &str, title: &str) -> FlowLogTemplate {
    FlowLogTemplate {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        timezone: "".to_string(),
        source: "".to_string(),
        status: FlowLogTemplateStatus::Created,
    }
}
