use crate::simulation_scale::SimulationScaleRun;
use crate::swarm::{SwarmBatchVsSingleMovementLoopMeasurementRun, SwarmLoadSmokeRun};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfHistoryScenarioFamily {
    SimulationScale,
    SwarmCollision,
    SwarmBatchVsSingle,
    GodotRender,
}

impl PerfHistoryScenarioFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SimulationScale => "simulation_scale",
            Self::SwarmCollision => "swarm_collision",
            Self::SwarmBatchVsSingle => "swarm_batch_vs_single",
            Self::GodotRender => "godot_render",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfHistoryStatus {
    Pass,
    Fail,
    Blocked,
    Informational,
}

impl PerfHistoryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Blocked => "blocked",
            Self::Informational => "informational",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfHistoryBudgetResult {
    Pass,
    Fail,
    Blocked,
}

impl PerfHistoryBudgetResult {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfHistoryClaimScope {
    InformationalContractOnly,
    LocalRegressionSignal,
}

impl PerfHistoryClaimScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InformationalContractOnly => "informational_contract_only",
            Self::LocalRegressionSignal => "local_regression_signal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfHistoryRedactionStatus {
    Pass,
    Fail,
    Blocked,
}

impl PerfHistoryRedactionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfHistoryPromotionDecision {
    Ready,
    Blocked,
}

impl PerfHistoryPromotionDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerfHistoryMetricPercentiles {
    pub p50: Option<f64>,
    pub p95: Option<f64>,
    pub p99: Option<f64>,
}

impl PerfHistoryMetricPercentiles {
    pub fn missing() -> Self {
        Self {
            p50: None,
            p95: None,
            p99: None,
        }
    }

    pub fn constant(value: f64) -> Self {
        Self {
            p50: Some(value),
            p95: Some(value),
            p99: Some(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerfHistoryMemoryMetrics {
    pub server_start: Option<f64>,
    pub server_peak: Option<f64>,
    pub server_end: Option<f64>,
}

impl PerfHistoryMemoryMetrics {
    pub fn missing() -> Self {
        Self {
            server_start: None,
            server_peak: None,
            server_end: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerfHistoryLocalElapsedMetrics {
    pub total: Option<f64>,
    pub spawn_ticks: Option<f64>,
    pub behavior: Option<f64>,
    pub movement_preview: Option<f64>,
    pub movement_tick: Option<f64>,
    pub configured_movement_tick: Option<f64>,
    pub configured_movement_loop: Option<f64>,
    pub static_obstacle_movement: Option<f64>,
    pub snapshot: Option<f64>,
    pub collision_diagnostics: Option<f64>,
}

impl PerfHistoryLocalElapsedMetrics {
    pub fn missing() -> Self {
        Self {
            total: None,
            spawn_ticks: None,
            behavior: None,
            movement_preview: None,
            movement_tick: None,
            configured_movement_tick: None,
            configured_movement_loop: None,
            static_obstacle_movement: None,
            snapshot: None,
            collision_diagnostics: None,
        }
    }

    pub fn swarm_collision_smoke(run: &SwarmLoadSmokeRun) -> Self {
        Self {
            total: Some(run.local_smoke_total_elapsed_us as f64),
            spawn_ticks: Some(run.spawn_ticks_elapsed_us as f64),
            behavior: Some(run.behavior_elapsed_us as f64),
            movement_preview: Some(run.movement_preview_elapsed_us as f64),
            movement_tick: Some(run.movement_tick_elapsed_us as f64),
            configured_movement_tick: Some(run.configured_movement_tick_elapsed_us as f64),
            configured_movement_loop: Some(run.configured_movement_loop_elapsed_us as f64),
            static_obstacle_movement: Some(run.static_obstacle_movement_elapsed_us as f64),
            snapshot: Some(run.snapshot_elapsed_us as f64),
            collision_diagnostics: Some(run.collision_diagnostics_elapsed_us as f64),
        }
    }

    pub fn swarm_batch_vs_single(run: &SwarmBatchVsSingleMovementLoopMeasurementRun) -> Self {
        Self {
            total: None,
            spawn_ticks: None,
            behavior: None,
            movement_preview: None,
            movement_tick: Some(run.single_elapsed_us_p95 as f64),
            configured_movement_tick: Some(run.batch_elapsed_us_p95 as f64),
            configured_movement_loop: Some(run.batch_to_single_elapsed_p95_bps as f64),
            static_obstacle_movement: None,
            snapshot: None,
            collision_diagnostics: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerfHistoryMetrics {
    pub server_tick_ms: PerfHistoryMetricPercentiles,
    pub snapshot_bytes: PerfHistoryMetricPercentiles,
    pub bandwidth_kb_s_per_client: PerfHistoryMetricPercentiles,
    pub memory_mb: PerfHistoryMemoryMetrics,
    pub local_elapsed_us: PerfHistoryLocalElapsedMetrics,
}

impl PerfHistoryMetrics {
    pub fn simulation_contract(snapshot_bytes: u64) -> Self {
        Self {
            server_tick_ms: PerfHistoryMetricPercentiles::missing(),
            snapshot_bytes: PerfHistoryMetricPercentiles::constant(snapshot_bytes as f64),
            bandwidth_kb_s_per_client: PerfHistoryMetricPercentiles::missing(),
            memory_mb: PerfHistoryMemoryMetrics::missing(),
            local_elapsed_us: PerfHistoryLocalElapsedMetrics::missing(),
        }
    }

    pub fn swarm_collision_smoke(run: &SwarmLoadSmokeRun) -> Self {
        Self {
            server_tick_ms: PerfHistoryMetricPercentiles::missing(),
            snapshot_bytes: PerfHistoryMetricPercentiles::constant(
                run.movement_tick_snapshot_bytes as f64,
            ),
            bandwidth_kb_s_per_client: PerfHistoryMetricPercentiles::constant(
                run.movement_tick_snapshot_bandwidth_kb_s_per_client,
            ),
            memory_mb: PerfHistoryMemoryMetrics::missing(),
            local_elapsed_us: PerfHistoryLocalElapsedMetrics::swarm_collision_smoke(run),
        }
    }

    pub fn swarm_batch_vs_single(run: &SwarmBatchVsSingleMovementLoopMeasurementRun) -> Self {
        Self {
            server_tick_ms: PerfHistoryMetricPercentiles::constant(
                run.batch_to_single_elapsed_p95_bps as f64,
            ),
            snapshot_bytes: PerfHistoryMetricPercentiles::missing(),
            bandwidth_kb_s_per_client: PerfHistoryMetricPercentiles::missing(),
            memory_mb: PerfHistoryMemoryMetrics::missing(),
            local_elapsed_us: PerfHistoryLocalElapsedMetrics::swarm_batch_vs_single(run),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PerfHistoryRow {
    pub schema_version: u16,
    pub ledger_id: String,
    pub date: String,
    pub machine_label: String,
    pub build_id: String,
    pub source_slice: String,
    pub scenario_id: String,
    pub scenario_family: PerfHistoryScenarioFamily,
    pub status: PerfHistoryStatus,
    pub budget_result: PerfHistoryBudgetResult,
    pub budget_keys: Vec<String>,
    pub metrics: PerfHistoryMetrics,
    pub source_artifact: String,
    pub why_changed: String,
    pub claim_scope: PerfHistoryClaimScope,
    pub redaction_status: PerfHistoryRedactionStatus,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwarmBatchVsSinglePromotionPolicy {
    pub min_rows: usize,
    pub max_batch_to_single_p95_bps: u32,
    pub max_single_elapsed_us_p95: u64,
    pub max_batch_elapsed_us_p95: u64,
}

impl Default for SwarmBatchVsSinglePromotionPolicy {
    fn default() -> Self {
        Self {
            min_rows: 3,
            max_batch_to_single_p95_bps: 12_000,
            max_single_elapsed_us_p95: 20_000_000,
            max_batch_elapsed_us_p95: 20_000_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwarmBatchVsSinglePromotionAssessment {
    pub row_count: usize,
    pub comparable_row_count: usize,
    pub min_rows_required: usize,
    pub max_batch_to_single_p95_bps: u32,
    pub max_single_elapsed_us_p95: u64,
    pub max_batch_elapsed_us_p95: u64,
    pub observed_batch_to_single_p95_bps_max: Option<u32>,
    pub observed_single_elapsed_us_p95_max: Option<u64>,
    pub observed_batch_elapsed_us_p95_max: Option<u64>,
    pub decision: PerfHistoryPromotionDecision,
    pub budget_result: PerfHistoryBudgetResult,
    pub claim_scope: PerfHistoryClaimScope,
    pub reason: &'static str,
}

impl PerfHistoryRow {
    pub fn simulation_scale_contract(
        date: impl Into<String>,
        machine_label: impl Into<String>,
        build_id: impl Into<String>,
        run: &SimulationScaleRun,
        source_artifact: impl Into<String>,
    ) -> Self {
        let date = date.into();
        let machine_label = machine_label.into();
        let build_id = build_id.into();
        let scenario_id = run.scenario_id.as_str().to_string();
        let ledger_id = build_ledger_id(&machine_label, &scenario_id, &date, &build_id);

        Self {
            schema_version: 1,
            ledger_id,
            date,
            machine_label,
            build_id,
            source_slice: "PHIST-02".to_string(),
            scenario_id,
            scenario_family: PerfHistoryScenarioFamily::SimulationScale,
            status: PerfHistoryStatus::Informational,
            budget_result: PerfHistoryBudgetResult::Blocked,
            budget_keys: vec![
                "server_tick_p99_ms_max".to_string(),
                "sim_scale_p95_ms_max".to_string(),
                "bandwidth_kb_s_per_client_p95_max".to_string(),
                "memory_rule".to_string(),
            ],
            metrics: PerfHistoryMetrics::simulation_contract(run.snapshot_bytes),
            source_artifact: source_artifact.into(),
            why_changed: "server simulation contract row emitted without measured timing"
                .to_string(),
            claim_scope: PerfHistoryClaimScope::InformationalContractOnly,
            redaction_status: PerfHistoryRedactionStatus::Pass,
            notes: format!(
                "entity_count={} tick_count={} occupied_cells={} final_tick={}",
                run.entity_count, run.tick_count, run.occupied_cells, run.final_tick
            ),
        }
    }

    pub fn swarm_collision_smoke(
        date: impl Into<String>,
        machine_label: impl Into<String>,
        build_id: impl Into<String>,
        run: &SwarmLoadSmokeRun,
        source_artifact: impl Into<String>,
    ) -> Self {
        let date = date.into();
        let machine_label = machine_label.into();
        let build_id = build_id.into();
        let scenario_id = "swarm_1k_server_behavior_collision_load".to_string();
        let ledger_id = build_ledger_id(&machine_label, &scenario_id, &date, &build_id);

        Self {
            schema_version: 1,
            ledger_id,
            date,
            machine_label,
            build_id,
            source_slice: "GSWARM-03".to_string(),
            scenario_id,
            scenario_family: PerfHistoryScenarioFamily::SwarmCollision,
            status: PerfHistoryStatus::Informational,
            budget_result: PerfHistoryBudgetResult::Blocked,
            budget_keys: vec![
                "swarm_local_smoke_elapsed_us_max".to_string(),
                "swarm_snapshot_bytes_max".to_string(),
                "swarm_bandwidth_kb_s_per_client_max".to_string(),
            ],
            metrics: PerfHistoryMetrics::swarm_collision_smoke(run),
            source_artifact: source_artifact.into(),
            why_changed: "swarm collision local smoke row emitted with stage elapsed-time counters"
                .to_string(),
            claim_scope: PerfHistoryClaimScope::LocalRegressionSignal,
            redaction_status: PerfHistoryRedactionStatus::Pass,
            notes: format!(
                "active_zombies={} tick_count={} movement_apply_samples={} collision_contacts={} total_elapsed_us={}",
                run.active_count,
                run.tick_count,
                run.movement_apply_sample_count,
                run.collision_contact_count,
                run.local_smoke_total_elapsed_us
            ),
        }
    }

    pub fn swarm_batch_vs_single_movement_loop(
        date: impl Into<String>,
        machine_label: impl Into<String>,
        build_id: impl Into<String>,
        run: &SwarmBatchVsSingleMovementLoopMeasurementRun,
        source_artifact: impl Into<String>,
    ) -> Self {
        let date = date.into();
        let machine_label = machine_label.into();
        let build_id = build_id.into();
        let scenario_id = "swarm_batch_vs_single_movement_loop_measurement".to_string();
        let ledger_id = build_ledger_id(&machine_label, &scenario_id, &date, &build_id);

        Self {
            schema_version: 1,
            ledger_id,
            date,
            machine_label,
            build_id,
            source_slice: "GSWARM-11".to_string(),
            scenario_id,
            scenario_family: PerfHistoryScenarioFamily::SwarmBatchVsSingle,
            status: PerfHistoryStatus::Informational,
            budget_result: PerfHistoryBudgetResult::Blocked,
            budget_keys: vec![
                "swarm_batch_vs_single_p95_ratio_bps_max".to_string(),
                "swarm_batch_movement_loop_p95_us_max".to_string(),
                "swarm_single_movement_loop_p95_us_max".to_string(),
            ],
            metrics: PerfHistoryMetrics::swarm_batch_vs_single(run),
            source_artifact: source_artifact.into(),
            why_changed:
                "swarm batch-vs-single movement loop row emitted with local p95 comparison"
                    .to_string(),
            claim_scope: PerfHistoryClaimScope::LocalRegressionSignal,
            redaction_status: PerfHistoryRedactionStatus::Pass,
            notes: format!(
                "active_zombies={} samples={} ticks_per_sample={} single_p95_us={} batch_p95_us={} batch_to_single_p95_bps={}",
                run.active_count,
                run.sample_count,
                run.tick_count_per_sample,
                run.single_elapsed_us_p95,
                run.batch_elapsed_us_p95,
                run.batch_to_single_elapsed_p95_bps
            ),
        }
    }

    pub fn to_json_line(&self) -> String {
        format!(
            "{{\"schema_version\":{},\"ledger_id\":{},\"date\":{},\"machine_label\":{},\"build_id\":{},\"source_slice\":{},\"scenario_id\":{},\"scenario_family\":{},\"status\":{},\"budget_result\":{},\"budget_keys\":{},\"metrics\":{},\"source_artifact\":{},\"why_changed\":{},\"claim_scope\":{},\"redaction_status\":{},\"notes\":{}}}",
            self.schema_version,
            json_string(&self.ledger_id),
            json_string(&self.date),
            json_string(&self.machine_label),
            json_string(&self.build_id),
            json_string(&self.source_slice),
            json_string(&self.scenario_id),
            json_string(self.scenario_family.as_str()),
            json_string(self.status.as_str()),
            json_string(self.budget_result.as_str()),
            json_string_array(&self.budget_keys),
            metrics_json(&self.metrics),
            json_string(&self.source_artifact),
            json_string(&self.why_changed),
            json_string(self.claim_scope.as_str()),
            json_string(self.redaction_status.as_str()),
            json_string(&self.notes)
        )
    }
}

pub fn assess_swarm_batch_vs_single_promotion(
    rows: &[PerfHistoryRow],
    policy: SwarmBatchVsSinglePromotionPolicy,
) -> SwarmBatchVsSinglePromotionAssessment {
    let mut comparable_row_count = 0;
    let mut observed_batch_to_single_p95_bps_max = None;
    let mut observed_single_elapsed_us_p95_max = None;
    let mut observed_batch_elapsed_us_p95_max = None;

    for row in rows.iter().filter(is_swarm_batch_vs_single_row) {
        let Some(batch_to_single_p95_bps) = batch_to_single_p95_bps(row) else {
            continue;
        };
        let Some(single_elapsed_us_p95) =
            positive_u64_metric(row.metrics.local_elapsed_us.movement_tick)
        else {
            continue;
        };
        let Some(batch_elapsed_us_p95) =
            positive_u64_metric(row.metrics.local_elapsed_us.configured_movement_tick)
        else {
            continue;
        };

        comparable_row_count += 1;
        observed_batch_to_single_p95_bps_max = Some(
            observed_batch_to_single_p95_bps_max
                .unwrap_or(batch_to_single_p95_bps)
                .max(batch_to_single_p95_bps),
        );
        observed_single_elapsed_us_p95_max = Some(
            observed_single_elapsed_us_p95_max
                .unwrap_or(single_elapsed_us_p95)
                .max(single_elapsed_us_p95),
        );
        observed_batch_elapsed_us_p95_max = Some(
            observed_batch_elapsed_us_p95_max
                .unwrap_or(batch_elapsed_us_p95)
                .max(batch_elapsed_us_p95),
        );
    }

    let (decision, budget_result, reason) = if comparable_row_count < policy.min_rows {
        (
            PerfHistoryPromotionDecision::Blocked,
            PerfHistoryBudgetResult::Blocked,
            "insufficient_comparable_rows",
        )
    } else if observed_batch_to_single_p95_bps_max
        .is_some_and(|observed| observed > policy.max_batch_to_single_p95_bps)
    {
        (
            PerfHistoryPromotionDecision::Blocked,
            PerfHistoryBudgetResult::Fail,
            "ratio_threshold_exceeded",
        )
    } else if observed_single_elapsed_us_p95_max
        .is_some_and(|observed| observed > policy.max_single_elapsed_us_p95)
    {
        (
            PerfHistoryPromotionDecision::Blocked,
            PerfHistoryBudgetResult::Fail,
            "single_elapsed_threshold_exceeded",
        )
    } else if observed_batch_elapsed_us_p95_max
        .is_some_and(|observed| observed > policy.max_batch_elapsed_us_p95)
    {
        (
            PerfHistoryPromotionDecision::Blocked,
            PerfHistoryBudgetResult::Fail,
            "batch_elapsed_threshold_exceeded",
        )
    } else {
        (
            PerfHistoryPromotionDecision::Ready,
            PerfHistoryBudgetResult::Pass,
            "enough_comparable_rows_under_local_thresholds",
        )
    };

    SwarmBatchVsSinglePromotionAssessment {
        row_count: rows.len(),
        comparable_row_count,
        min_rows_required: policy.min_rows,
        max_batch_to_single_p95_bps: policy.max_batch_to_single_p95_bps,
        max_single_elapsed_us_p95: policy.max_single_elapsed_us_p95,
        max_batch_elapsed_us_p95: policy.max_batch_elapsed_us_p95,
        observed_batch_to_single_p95_bps_max,
        observed_single_elapsed_us_p95_max,
        observed_batch_elapsed_us_p95_max,
        decision,
        budget_result,
        claim_scope: PerfHistoryClaimScope::LocalRegressionSignal,
        reason,
    }
}

fn is_swarm_batch_vs_single_row(row: &&PerfHistoryRow) -> bool {
    row.scenario_family == PerfHistoryScenarioFamily::SwarmBatchVsSingle
        && row.scenario_id == "swarm_batch_vs_single_movement_loop_measurement"
        && row.claim_scope == PerfHistoryClaimScope::LocalRegressionSignal
        && row.redaction_status == PerfHistoryRedactionStatus::Pass
}

fn batch_to_single_p95_bps(row: &PerfHistoryRow) -> Option<u32> {
    positive_u32_metric(row.metrics.server_tick_ms.p95)
        .or_else(|| positive_u32_metric(row.metrics.local_elapsed_us.configured_movement_loop))
}

fn positive_u32_metric(value: Option<f64>) -> Option<u32> {
    let value = value?;
    if !value.is_finite() || value <= 0.0 || value > u32::MAX as f64 {
        return None;
    }
    Some(value.round() as u32)
}

fn positive_u64_metric(value: Option<f64>) -> Option<u64> {
    let value = value?;
    if !value.is_finite() || value <= 0.0 || value > u64::MAX as f64 {
        return None;
    }
    Some(value.round() as u64)
}

pub fn build_ledger_id(
    machine_label: &str,
    scenario_id: &str,
    date: &str,
    build_id: &str,
) -> String {
    format!("{machine_label}__{scenario_id}__{date}__{build_id}")
}

fn metrics_json(metrics: &PerfHistoryMetrics) -> String {
    format!(
        "{{\"server_tick_ms\":{},\"snapshot_bytes\":{},\"bandwidth_kb_s_per_client\":{},\"memory_mb\":{},\"local_elapsed_us\":{}}}",
        percentile_json(&metrics.server_tick_ms),
        percentile_json(&metrics.snapshot_bytes),
        percentile_json(&metrics.bandwidth_kb_s_per_client),
        memory_json(&metrics.memory_mb),
        local_elapsed_json(&metrics.local_elapsed_us)
    )
}

fn percentile_json(metric: &PerfHistoryMetricPercentiles) -> String {
    format!(
        "{{\"p50\":{},\"p95\":{},\"p99\":{}}}",
        json_optional_number(metric.p50),
        json_optional_number(metric.p95),
        json_optional_number(metric.p99)
    )
}

fn memory_json(metric: &PerfHistoryMemoryMetrics) -> String {
    format!(
        "{{\"server_start\":{},\"server_peak\":{},\"server_end\":{}}}",
        json_optional_number(metric.server_start),
        json_optional_number(metric.server_peak),
        json_optional_number(metric.server_end)
    )
}

fn local_elapsed_json(metric: &PerfHistoryLocalElapsedMetrics) -> String {
    format!(
        "{{\"total\":{},\"spawn_ticks\":{},\"behavior\":{},\"movement_preview\":{},\"movement_tick\":{},\"configured_movement_tick\":{},\"configured_movement_loop\":{},\"static_obstacle_movement\":{},\"snapshot\":{},\"collision_diagnostics\":{}}}",
        json_optional_number(metric.total),
        json_optional_number(metric.spawn_ticks),
        json_optional_number(metric.behavior),
        json_optional_number(metric.movement_preview),
        json_optional_number(metric.movement_tick),
        json_optional_number(metric.configured_movement_tick),
        json_optional_number(metric.configured_movement_loop),
        json_optional_number(metric.static_obstacle_movement),
        json_optional_number(metric.snapshot),
        json_optional_number(metric.collision_diagnostics)
    )
}

fn json_optional_number(value: Option<f64>) -> String {
    match value {
        Some(value) if value.is_finite() => value.to_string(),
        _ => "null".to_string(),
    }
}

fn json_string_array(values: &[String]) -> String {
    let encoded = values
        .iter()
        .map(|value| json_string(value))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{encoded}]")
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perf_budget::BudgetResult;
    use crate::simulation_scale::{run_simulation_scale_scenario, SIMULATION_SCALE_SCENARIOS};
    use crate::swarm::{run_swarm_batch_vs_single_movement_loop_measurement, run_swarm_load_smoke};

    #[test]
    fn perf_history_simulation_contract_row_uses_schema_fields() {
        let run = run_simulation_scale_scenario(SIMULATION_SCALE_SCENARIOS[0]);
        let row = PerfHistoryRow::simulation_scale_contract(
            "2026-07-03",
            "local-dev-01",
            "local-uncommitted",
            &run,
            "tests/perf/sim-scale-local-report.json",
        );

        assert_eq!(
            row.ledger_id,
            "local-dev-01__sim_1k_single_client__2026-07-03__local-uncommitted"
        );
        assert_eq!(row.schema_version, 1);
        assert_eq!(row.source_slice, "PHIST-02");
        assert_eq!(
            row.scenario_family,
            PerfHistoryScenarioFamily::SimulationScale
        );
        assert_eq!(row.status, PerfHistoryStatus::Informational);
        assert_eq!(row.budget_result, PerfHistoryBudgetResult::Blocked);
        assert_eq!(
            row.claim_scope,
            PerfHistoryClaimScope::InformationalContractOnly
        );
        assert_eq!(row.redaction_status, PerfHistoryRedactionStatus::Pass);
        assert!(row.metrics.snapshot_bytes.p95.unwrap() > 1_000.0);
        assert!(row.metrics.server_tick_ms.p95.is_none());
        assert!(row.metrics.local_elapsed_us.total.is_none());
    }

    #[test]
    fn perf_history_row_emits_json_line_without_private_paths() {
        let run = run_simulation_scale_scenario(SIMULATION_SCALE_SCENARIOS[0]);
        let row = PerfHistoryRow::simulation_scale_contract(
            "2026-07-03",
            "local-dev-01",
            "local-uncommitted",
            &run,
            "tests/perf/sim-scale-local-report.json",
        );
        let json_line = row.to_json_line();

        assert!(json_line.starts_with("{\"schema_version\":1"));
        assert!(json_line.contains(
            "\"ledger_id\":\"local-dev-01__sim_1k_single_client__2026-07-03__local-uncommitted\""
        ));
        assert!(json_line.contains("\"scenario_family\":\"simulation_scale\""));
        assert!(json_line.contains("\"budget_result\":\"blocked\""));
        assert!(json_line.contains("\"redaction_status\":\"pass\""));
        assert!(json_line.contains("\"metrics\":{\"server_tick_ms\""));
        assert!(json_line.contains("\"local_elapsed_us\""));
        assert!(json_line.contains("\"total\":null"));
        assert!(!json_line.contains("C:\\"));
    }

    #[test]
    fn perf_history_swarm_collision_row_records_local_elapsed_stages() {
        let run = run_swarm_load_smoke();
        let row = PerfHistoryRow::swarm_collision_smoke(
            "2026-07-05",
            "local-dev-01",
            "local-uncommitted",
            &run,
            "tests/perf/swarm-load-smoke-report.json",
        );

        assert_eq!(
            row.ledger_id,
            "local-dev-01__swarm_1k_server_behavior_collision_load__2026-07-05__local-uncommitted"
        );
        assert_eq!(row.source_slice, "GSWARM-03");
        assert_eq!(
            row.scenario_family,
            PerfHistoryScenarioFamily::SwarmCollision
        );
        assert_eq!(row.status, PerfHistoryStatus::Informational);
        assert_eq!(row.budget_result, PerfHistoryBudgetResult::Blocked);
        assert_eq!(
            row.claim_scope,
            PerfHistoryClaimScope::LocalRegressionSignal
        );
        assert_eq!(row.metrics.snapshot_bytes.p95, Some(36_024.0));
        assert!(row.metrics.bandwidth_kb_s_per_client.p95.unwrap() > 0.0);
        assert!(row.metrics.server_tick_ms.p95.is_none());
        assert!(row.metrics.local_elapsed_us.total.unwrap() > 0.0);
        assert!(row.metrics.local_elapsed_us.movement_tick.unwrap() > 0.0);
        assert!(
            row.metrics
                .local_elapsed_us
                .configured_movement_loop
                .unwrap()
                > 0.0
        );
        assert!(row.metrics.local_elapsed_us.collision_diagnostics.unwrap() > 0.0);
        assert!(row.notes.contains("active_zombies=1000"));
    }

    #[test]
    fn perf_history_swarm_collision_json_line_is_redacted_and_machine_readable() {
        let run = run_swarm_load_smoke();
        let row = PerfHistoryRow::swarm_collision_smoke(
            "2026-07-05",
            "local-dev-01",
            "local-uncommitted",
            &run,
            "tests/perf/swarm-load-smoke-report.json",
        );
        let json_line = row.to_json_line();

        assert!(json_line.contains("\"scenario_family\":\"swarm_collision\""));
        assert!(json_line.contains("\"claim_scope\":\"local_regression_signal\""));
        assert!(json_line.contains("\"local_elapsed_us\":{\"total\":"));
        assert!(json_line.contains("\"movement_tick\":"));
        assert!(json_line.contains("\"configured_movement_loop\":"));
        assert!(
            json_line.contains("\"source_artifact\":\"tests/perf/swarm-load-smoke-report.json\"")
        );
        assert!(!json_line.contains("C:\\"));
    }

    #[test]
    fn perf_history_swarm_batch_vs_single_row_records_comparison_metrics() {
        let run = run_swarm_batch_vs_single_movement_loop_measurement(3);
        let row = PerfHistoryRow::swarm_batch_vs_single_movement_loop(
            "2026-07-05",
            "local-dev-01",
            "local-uncommitted",
            &run,
            "tests/perf/swarm-batch-vs-single-movement-loop-report.json",
        );

        assert_eq!(
            row.ledger_id,
            "local-dev-01__swarm_batch_vs_single_movement_loop_measurement__2026-07-05__local-uncommitted"
        );
        assert_eq!(row.source_slice, "GSWARM-11");
        assert_eq!(
            row.scenario_family,
            PerfHistoryScenarioFamily::SwarmBatchVsSingle
        );
        assert_eq!(row.status, PerfHistoryStatus::Informational);
        assert_eq!(row.budget_result, PerfHistoryBudgetResult::Blocked);
        assert_eq!(
            row.claim_scope,
            PerfHistoryClaimScope::LocalRegressionSignal
        );
        assert!(row
            .budget_keys
            .contains(&"swarm_batch_vs_single_p95_ratio_bps_max".to_string()));
        assert_eq!(
            row.metrics.server_tick_ms.p95,
            Some(run.batch_to_single_elapsed_p95_bps as f64)
        );
        assert_eq!(
            row.metrics.local_elapsed_us.movement_tick,
            Some(run.single_elapsed_us_p95 as f64)
        );
        assert_eq!(
            row.metrics.local_elapsed_us.configured_movement_tick,
            Some(run.batch_elapsed_us_p95 as f64)
        );
        assert_eq!(
            row.metrics.local_elapsed_us.configured_movement_loop,
            Some(run.batch_to_single_elapsed_p95_bps as f64)
        );
        assert!(row.metrics.snapshot_bytes.p95.is_none());
        assert!(row.notes.contains("active_zombies=1000"));
        assert!(row.notes.contains("batch_to_single_p95_bps="));
    }

    #[test]
    fn perf_history_swarm_batch_vs_single_json_line_is_redacted_and_machine_readable() {
        let run = run_swarm_batch_vs_single_movement_loop_measurement(3);
        let row = PerfHistoryRow::swarm_batch_vs_single_movement_loop(
            "2026-07-05",
            "local-dev-01",
            "local-uncommitted",
            &run,
            "tests/perf/swarm-batch-vs-single-movement-loop-report.json",
        );
        let json_line = row.to_json_line();

        assert!(json_line.contains("\"scenario_family\":\"swarm_batch_vs_single\""));
        assert!(json_line.contains("\"source_slice\":\"GSWARM-11\""));
        assert!(json_line.contains("\"claim_scope\":\"local_regression_signal\""));
        assert!(json_line.contains("\"server_tick_ms\":{\"p50\":"));
        assert!(json_line.contains("\"configured_movement_tick\":"));
        assert!(json_line.contains("\"configured_movement_loop\":"));
        assert!(json_line.contains(
            "\"source_artifact\":\"tests/perf/swarm-batch-vs-single-movement-loop-report.json\""
        ));
        assert!(!json_line.contains("C:\\"));
    }

    #[test]
    fn perf_history_swarm_batch_vs_single_promotion_blocks_until_enough_rows() {
        let run = synthetic_batch_vs_single_run(8_500, 4_200_000, 3_600_000);
        let row = PerfHistoryRow::swarm_batch_vs_single_movement_loop(
            "2026-07-05",
            "local-dev-01",
            "local-uncommitted",
            &run,
            "tests/perf/swarm-batch-vs-single-movement-loop-report.json",
        );

        let assessment = assess_swarm_batch_vs_single_promotion(
            &[row],
            SwarmBatchVsSinglePromotionPolicy::default(),
        );

        assert_eq!(assessment.row_count, 1);
        assert_eq!(assessment.comparable_row_count, 1);
        assert_eq!(assessment.min_rows_required, 3);
        assert_eq!(assessment.decision, PerfHistoryPromotionDecision::Blocked);
        assert_eq!(assessment.budget_result, PerfHistoryBudgetResult::Blocked);
        assert_eq!(assessment.reason, "insufficient_comparable_rows");
        assert_eq!(
            assessment.claim_scope,
            PerfHistoryClaimScope::LocalRegressionSignal
        );
        assert_eq!(assessment.observed_batch_to_single_p95_bps_max, Some(8_500));
    }

    #[test]
    fn perf_history_swarm_batch_vs_single_promotion_ready_after_three_local_rows() {
        let rows = vec![
            synthetic_batch_vs_single_row(
                "2026-07-05",
                "local-build-a",
                8_500,
                4_200_000,
                3_600_000,
            ),
            synthetic_batch_vs_single_row(
                "2026-07-06",
                "local-build-b",
                8_700,
                4_000_000,
                3_700_000,
            ),
            synthetic_batch_vs_single_row(
                "2026-07-07",
                "local-build-c",
                9_100,
                4_100_000,
                3_900_000,
            ),
        ];

        let assessment = assess_swarm_batch_vs_single_promotion(
            &rows,
            SwarmBatchVsSinglePromotionPolicy::default(),
        );

        assert_eq!(assessment.comparable_row_count, 3);
        assert_eq!(assessment.decision, PerfHistoryPromotionDecision::Ready);
        assert_eq!(assessment.budget_result, PerfHistoryBudgetResult::Pass);
        assert_eq!(
            assessment.reason,
            "enough_comparable_rows_under_local_thresholds"
        );
        assert_eq!(assessment.observed_batch_to_single_p95_bps_max, Some(9_100));
        assert_eq!(
            assessment.observed_single_elapsed_us_p95_max,
            Some(4_200_000)
        );
        assert_eq!(
            assessment.observed_batch_elapsed_us_p95_max,
            Some(3_900_000)
        );
    }

    #[test]
    fn perf_history_swarm_batch_vs_single_promotion_blocks_ratio_regression() {
        let rows = vec![
            synthetic_batch_vs_single_row(
                "2026-07-05",
                "local-build-a",
                8_500,
                4_200_000,
                3_600_000,
            ),
            synthetic_batch_vs_single_row(
                "2026-07-06",
                "local-build-b",
                12_001,
                4_000_000,
                4_800_000,
            ),
            synthetic_batch_vs_single_row(
                "2026-07-07",
                "local-build-c",
                9_100,
                4_100_000,
                3_900_000,
            ),
        ];

        let assessment = assess_swarm_batch_vs_single_promotion(
            &rows,
            SwarmBatchVsSinglePromotionPolicy::default(),
        );

        assert_eq!(assessment.comparable_row_count, 3);
        assert_eq!(assessment.decision, PerfHistoryPromotionDecision::Blocked);
        assert_eq!(assessment.budget_result, PerfHistoryBudgetResult::Fail);
        assert_eq!(assessment.reason, "ratio_threshold_exceeded");
        assert_eq!(
            assessment.observed_batch_to_single_p95_bps_max,
            Some(12_001)
        );
    }

    fn synthetic_batch_vs_single_row(
        date: &str,
        build_id: &str,
        batch_to_single_elapsed_p95_bps: u32,
        single_elapsed_us_p95: u64,
        batch_elapsed_us_p95: u64,
    ) -> PerfHistoryRow {
        let run = synthetic_batch_vs_single_run(
            batch_to_single_elapsed_p95_bps,
            single_elapsed_us_p95,
            batch_elapsed_us_p95,
        );
        PerfHistoryRow::swarm_batch_vs_single_movement_loop(
            date,
            "local-dev-01",
            build_id,
            &run,
            "tests/perf/swarm-batch-vs-single-movement-loop-report.json",
        )
    }

    fn synthetic_batch_vs_single_run(
        batch_to_single_elapsed_p95_bps: u32,
        single_elapsed_us_p95: u64,
        batch_elapsed_us_p95: u64,
    ) -> SwarmBatchVsSingleMovementLoopMeasurementRun {
        SwarmBatchVsSingleMovementLoopMeasurementRun {
            sample_count: 3,
            tick_count_per_sample: 2,
            movement_sample_limit: 2,
            active_count: 1000,
            single_elapsed_us_p50: single_elapsed_us_p95.saturating_sub(100_000),
            single_elapsed_us_p95,
            single_elapsed_us_p99: single_elapsed_us_p95.saturating_add(100_000),
            batch_elapsed_us_p50: batch_elapsed_us_p95.saturating_sub(100_000),
            batch_elapsed_us_p95,
            batch_elapsed_us_p99: batch_elapsed_us_p95.saturating_add(100_000),
            batch_to_single_elapsed_p95_bps,
            single_movement_sample_count_total: 12,
            batch_movement_sample_count_total: 12,
            single_applied_delta_count_total: 12,
            batch_applied_delta_count_total: 12,
            single_physics_iterations_run_total: 24,
            batch_physics_iterations_run_total: 24,
            single_flow_field_cache_hit_count_total: 6,
            batch_flow_field_cache_hit_count_total: 6,
            single_flow_field_cache_eviction_count_total: 0,
            batch_flow_field_cache_eviction_count_total: 0,
            single_moved_entity_count_min: 1,
            batch_moved_entity_count_min: 1,
            budget_result: BudgetResult::Blocked,
            claim_scope: "local_batch_vs_single_measured_harness_only",
        }
    }
}
