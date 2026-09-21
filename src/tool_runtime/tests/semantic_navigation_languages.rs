//! Focused, process-free tests of startup language selection and availability.
use super::*;
use crate::lsp_bridge::LspServerStatusEntry;

fn status_for(language: &str, provider: &str, status: LspAvailabilityStatus) -> LspStatusResult {
    LspStatusResult {
        project: "demo".to_string(),
        detected_languages: vec![language.to_string()],
        servers: vec![LspServerStatusEntry {
            language: language.to_string(),
            server: provider.to_string(),
            available: status != LspAvailabilityStatus::Unavailable,
            running: status == LspAvailabilityStatus::Running,
            status,
            source: None,
            position_encoding: if status == LspAvailabilityStatus::Running {
                Some("utf-16".to_string())
            } else {
                None
            },
        }],
        warnings: Vec::new(),
    }
}

#[test]
fn python_and_typescript_startup_preserves_observed_availability_states() {
    for (language, provider, limitation) in [
        (PYTHON_LANGUAGE, PYRIGHT_SERVER, "python_only"),
        (TYPESCRIPT_LANGUAGE, TYPESCRIPT_SERVER, "typescript_only"),
    ] {
        for (state, expected, available, recommended) in [
            (
                LspAvailabilityStatus::Available,
                SemanticNavigationStartupStatus::Available,
                true,
                true,
            ),
            (
                LspAvailabilityStatus::Running,
                SemanticNavigationStartupStatus::Running,
                true,
                true,
            ),
            (
                LspAvailabilityStatus::Initializing,
                SemanticNavigationStartupStatus::Initializing,
                true,
                false,
            ),
            (
                LspAvailabilityStatus::Unavailable,
                SemanticNavigationStartupStatus::Unavailable,
                false,
                false,
            ),
            (
                LspAvailabilityStatus::Crashed,
                SemanticNavigationStartupStatus::Crashed,
                true,
                false,
            ),
        ] {
            let result = SemanticNavigationStartupSummary::from_lsp_status(
                status_for(language, provider, state),
                "demo",
            )
            .unwrap();
            assert_eq!(result.language, Some(language));
            assert_eq!(result.server, Some(provider));
            assert_eq!(result.status, expected);
            assert_eq!(result.available, Some(available));
            assert_eq!(result.recommended, recommended);
            assert_eq!(result.limitations[0], limitation);
            assert!(result.tools.contains(&"document_symbols"));
            assert_eq!(
                result.position_encoding.as_deref(),
                if state == LspAvailabilityStatus::Running {
                    Some("utf-16")
                } else {
                    None
                }
            );
        }
        let mut crashed = status_for(language, provider, LspAvailabilityStatus::Crashed);
        crashed.servers[0].available = false;
        let result = SemanticNavigationStartupSummary::from_lsp_status(crashed, "demo").unwrap();
        assert_eq!(result.available, Some(false));
        assert!(!result.recommended);
    }
}

#[test]
fn startup_language_selection_keeps_rust_go_precedence_and_python_before_typescript() {
    let profiles = [
        (RUST_LANGUAGE, RUST_ANALYZER_SERVER),
        (GO_LANGUAGE, GOPLS_SERVER),
        (PYTHON_LANGUAGE, PYRIGHT_SERVER),
        (TYPESCRIPT_LANGUAGE, TYPESCRIPT_SERVER),
    ];
    for first in 0..profiles.len() {
        let mut status = LspStatusResult {
            project: "demo".to_string(),
            detected_languages: Vec::new(),
            servers: Vec::new(),
            warnings: Vec::new(),
        };
        // Reverse source order to prove selection is deterministic, not provider order.
        for &(language, provider) in profiles[first..].iter().rev() {
            let entry = status_for(language, provider, LspAvailabilityStatus::Available);
            status.detected_languages.extend(entry.detected_languages);
            status.servers.extend(entry.servers);
        }
        let summary = SemanticNavigationStartupSummary::from_lsp_status(status, "demo").unwrap();
        assert_eq!(summary.language, Some(profiles[first].0));
        assert_eq!(summary.server, Some(profiles[first].1));
    }
}

#[test]
fn python_and_typescript_missing_or_mismatched_status_fails_closed() {
    for (language, provider) in [
        (PYTHON_LANGUAGE, PYRIGHT_SERVER),
        (TYPESCRIPT_LANGUAGE, TYPESCRIPT_SERVER),
    ] {
        let valid = status_for(language, provider, LspAvailabilityStatus::Available);
        let mut missing = valid.clone();
        missing.servers.clear();
        let mut wrong_provider = valid.clone();
        wrong_provider.servers[0].server = "unexpected-provider".to_string();
        let mut wrong_project = valid.clone();
        wrong_project.project = "another-project".to_string();
        let mut invalid_encoding = valid;
        invalid_encoding.servers[0].position_encoding = Some("invalid-encoding".to_string());
        for status in [missing, wrong_provider, wrong_project, invalid_encoding] {
            assert_eq!(
                SemanticNavigationStartupSummary::from_lsp_status(status, "demo"),
                Err(SemanticNavigationReasonCode::MalformedRunnerResult),
            );
        }
    }
}

#[test]
fn extending_languages_does_not_promote_timeouts_to_available() {
    let summary = SemanticNavigationStartupSummary::probe_timeout();
    assert_eq!(summary.available, None);
    assert_eq!(summary.language, None);
    assert_eq!(summary.server, None);
    assert!(!summary.recommended);
    assert_eq!(
        summary.status,
        SemanticNavigationStartupStatus::ProbeTimeout
    );
    assert_eq!(
        summary.reason_code,
        Some(SemanticNavigationReasonCode::StatusProbeTimedOut)
    );
    assert_eq!(
        DEFAULT_SEMANTIC_NAVIGATION_PROBE_TIMEOUT,
        Duration::from_secs(2)
    );
}
