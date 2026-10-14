use super::*;

/// How screens and exports show a trust domain template.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTemplateView {
    pub code: String,
    pub label_text: String,
    pub attempts_text: String,
    pub budget_text: String,
    pub host: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_trust_domain_template_view(item: &TrustDomainTemplate) -> TrustDomainTemplateView {
    TrustDomainTemplateView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        attempts_text: item.attempts.to_string(),
        budget_text: format!("{:.2}", item.budget),
        host: item.host.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_template_views(items: &[TrustDomainTemplate]) -> Vec<TrustDomainTemplateView> {
    let mut views: Vec<TrustDomainTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_template_view(item));
    }
    views
}

pub fn sum_trust_domain_template_view(view: &TrustDomainTemplateView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_budget + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_template("code-668", "label-478");
        let view = to_trust_domain_template_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_template_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
