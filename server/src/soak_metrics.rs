#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SoakMetric {
    MemoryServerStartMb,
    MemoryServerPeakMb,
    MemoryServerEndMb,
    AllocationBytesTotal,
    QueueDepthMax,
    ConnectionsActive,
    SnapshotsDropped,
    ResendsQueued,
    ResendsSent,
    ShutdownClean,
}

impl SoakMetric {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MemoryServerStartMb => "memory_server_start_mb",
            Self::MemoryServerPeakMb => "memory_server_peak_mb",
            Self::MemoryServerEndMb => "memory_server_end_mb",
            Self::AllocationBytesTotal => "allocation_bytes_total",
            Self::QueueDepthMax => "queue_depth_max",
            Self::ConnectionsActive => "connections_active",
            Self::SnapshotsDropped => "snapshots_dropped",
            Self::ResendsQueued => "resends_queued",
            Self::ResendsSent => "resends_sent",
            Self::ShutdownClean => "shutdown_clean",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoakMetricSample {
    pub metric: SoakMetric,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoakMetrics {
    values: Vec<SoakMetricSample>,
}

impl SoakMetrics {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn set(&mut self, metric: SoakMetric, value: u64) {
        if let Some(sample) = self
            .values
            .iter_mut()
            .find(|sample| sample.metric == metric)
        {
            sample.value = value;
            return;
        }

        self.values.push(SoakMetricSample { metric, value });
        self.values.sort_by_key(|sample| sample.metric);
    }

    pub fn add(&mut self, metric: SoakMetric, delta: u64) -> u64 {
        let next = self.get(metric).saturating_add(delta);
        self.set(metric, next);
        next
    }

    pub fn get(&self, metric: SoakMetric) -> u64 {
        self.values
            .iter()
            .find(|sample| sample.metric == metric)
            .map(|sample| sample.value)
            .unwrap_or(0)
    }

    pub fn record_shutdown(&mut self, clean: bool) {
        self.set(SoakMetric::ShutdownClean, u64::from(clean));
    }

    pub fn snapshot(&self) -> Vec<SoakMetricSample> {
        self.values.clone()
    }

    pub fn snapshot_pairs(&self) -> Vec<(&'static str, u64)> {
        self.values
            .iter()
            .map(|sample| (sample.metric.as_str(), sample.value))
            .collect()
    }

    pub fn has_required_foundation_metrics(&self) -> bool {
        REQUIRED_SOAK_METRICS
            .iter()
            .all(|metric| self.values.iter().any(|sample| sample.metric == *metric))
    }
}

impl Default for SoakMetrics {
    fn default() -> Self {
        Self::new()
    }
}

pub const REQUIRED_SOAK_METRICS: [SoakMetric; 10] = [
    SoakMetric::MemoryServerStartMb,
    SoakMetric::MemoryServerPeakMb,
    SoakMetric::MemoryServerEndMb,
    SoakMetric::AllocationBytesTotal,
    SoakMetric::QueueDepthMax,
    SoakMetric::ConnectionsActive,
    SoakMetric::SnapshotsDropped,
    SoakMetric::ResendsQueued,
    SoakMetric::ResendsSent,
    SoakMetric::ShutdownClean,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soak_metrics_emit_stable_snapshot_pairs() {
        let mut metrics = SoakMetrics::new();
        metrics.set(SoakMetric::MemoryServerStartMb, 32);
        metrics.set(SoakMetric::MemoryServerPeakMb, 48);
        metrics.set(SoakMetric::MemoryServerEndMb, 36);
        metrics.set(SoakMetric::AllocationBytesTotal, 4096);
        metrics.set(SoakMetric::QueueDepthMax, 7);
        metrics.set(SoakMetric::ConnectionsActive, 2);
        metrics.add(SoakMetric::SnapshotsDropped, 3);
        metrics.add(SoakMetric::ResendsQueued, 5);
        metrics.add(SoakMetric::ResendsSent, 4);
        metrics.record_shutdown(true);

        assert!(metrics.has_required_foundation_metrics());
        assert_eq!(metrics.get(SoakMetric::ShutdownClean), 1);
        assert_eq!(
            metrics.snapshot_pairs(),
            vec![
                ("memory_server_start_mb", 32),
                ("memory_server_peak_mb", 48),
                ("memory_server_end_mb", 36),
                ("allocation_bytes_total", 4096),
                ("queue_depth_max", 7),
                ("connections_active", 2),
                ("snapshots_dropped", 3),
                ("resends_queued", 5),
                ("resends_sent", 4),
                ("shutdown_clean", 1),
            ]
        );
    }

    #[test]
    fn soak_metrics_report_missing_required_values() {
        let mut metrics = SoakMetrics::new();
        metrics.set(SoakMetric::ConnectionsActive, 2);

        assert!(!metrics.has_required_foundation_metrics());
    }
}
