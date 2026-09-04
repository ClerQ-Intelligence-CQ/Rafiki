//! Rafiki SAE: signal acquisition engine.
//!
//! Public re-anchor of the Teka acquisition path. Raw sensor input
//! (accelerometer, GPS, barometer, microphone amplitude envelope,
//! screen state) normalizes into a clean typed event stream with
//! sampling-rate control and power-aware duty cycling. Raw sensor data
//! never persists; only the typed events leave this crate.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Accelerometer {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GpsReading {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_m: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Barometer {
    pub pressure_hpa: f32,
    pub temperature_c: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MicAmplitude {
    pub envelope_db: f32,
    pub peak_hz: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScreenState {
    pub on: bool,
    pub brightness_pct: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorType {
    Accelerometer,
    Gps,
    Barometer,
    MicAmplitude,
    ScreenState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SensorData {
    Accelerometer(Accelerometer),
    Gps(GpsReading),
    Barometer(Barometer),
    MicAmplitude(MicAmplitude),
    ScreenState(ScreenState),
}

impl SensorData {
    pub fn accel(x: f32, y: f32, z: f32) -> Self {
        SensorData::Accelerometer(Accelerometer { x, y, z })
    }
    pub fn gps(lat: f64, lng: f64, acc: f32) -> Self {
        SensorData::Gps(GpsReading { latitude: lat, longitude: lng, accuracy_m: acc })
    }
    pub fn baro(pressure: f32, temp: f32) -> Self {
        SensorData::Barometer(Barometer { pressure_hpa: pressure, temperature_c: temp })
    }
    pub fn mic(envelope: f32, peak: f32) -> Self {
        SensorData::MicAmplitude(MicAmplitude { envelope_db: envelope, peak_hz: peak })
    }
    pub fn screen(on: bool, brightness: u8) -> Self {
        SensorData::ScreenState(ScreenState { on, brightness_pct: brightness })
    }
    pub fn sensor_type(&self) -> SensorType {
        match self {
            SensorData::Accelerometer(_) => SensorType::Accelerometer,
            SensorData::Gps(_) => SensorType::Gps,
            SensorData::Barometer(_) => SensorType::Barometer,
            SensorData::MicAmplitude(_) => SensorType::MicAmplitude,
            SensorData::ScreenState(_) => SensorType::ScreenState,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorEvent {
    pub timestamp_ms: u128,
    pub sensor_type: SensorType,
    pub data: SensorData,
    pub confidence: f32,
}

/// Confidence derives from signal quality per sensor. Documented, no
/// magic constants: GPS degrades with reported accuracy, microphone
/// with envelope level, accelerometer with deviation from rest
/// gravity, barometer with plausible range, screen state is boolean.
pub fn confidence_for(sensor_type: SensorType, data: &SensorData) -> f32 {
    match (sensor_type, data) {
        (SensorType::Gps, SensorData::Gps(g)) => (1.0 - g.accuracy_m / 100.0).clamp(0.0, 0.95),
        (SensorType::MicAmplitude, SensorData::MicAmplitude(m)) => {
            (0.5 + (m.envelope_db / 80.0) * 0.45).clamp(0.0, 0.95)
        }
        (SensorType::Accelerometer, SensorData::Accelerometer(a)) => {
            let mag = (a.x * a.x + a.y * a.y + a.z * a.z).sqrt();
            (1.0 - (mag - 9.81).abs() / 20.0).clamp(0.0, 0.95)
        }
        (SensorType::Barometer, SensorData::Barometer(b)) => {
            if (950.0..=1050.0).contains(&b.pressure_hpa) {
                0.9
            } else {
                0.6
            }
        }
        (SensorType::ScreenState, _) => 0.9,
        _ => 0.5,
    }
}

#[derive(Debug, Clone)]
pub struct SamplingConfig {
    pub min_interval_ms: u64,
    pub max_quiet_interval_ms: u64,
    pub active_interval_ms: u64,
    pub activity_threshold: f32,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            min_interval_ms: 10,
            max_quiet_interval_ms: 500,
            active_interval_ms: 50,
            activity_threshold: 0.05,
        }
    }
}

#[derive(Debug, Default)]
pub struct ActivityDetector {
    last_accel: Option<Accelerometer>,
    last_gps: Option<GpsReading>,
    last_mic: Option<MicAmplitude>,
}

impl ActivityDetector {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_active(&mut self, event: &SensorEvent, threshold: f32) -> bool {
        match &event.data {
            SensorData::Accelerometer(a) => {
                if let Some(last) = self.last_accel {
                    let dx = (a.x - last.x).abs();
                    let dy = (a.y - last.y).abs();
                    let dz = (a.z - last.z).abs();
                    if dx > threshold || dy > threshold || dz > threshold {
                        self.last_accel = Some(*a);
                        return true;
                    }
                } else {
                    self.last_accel = Some(*a);
                    return true;
                }
            }
            SensorData::Gps(g) => {
                if let Some(last) = self.last_gps {
                    let dlat = (g.latitude - last.latitude).abs();
                    let dlng = (g.longitude - last.longitude).abs();
                    if dlat > threshold as f64 || dlng > threshold as f64 {
                        self.last_gps = Some(*g);
                        return true;
                    }
                } else {
                    self.last_gps = Some(*g);
                    return true;
                }
            }
            SensorData::MicAmplitude(m) => {
                if let Some(last) = self.last_mic {
                    if (m.envelope_db - last.envelope_db).abs() > threshold {
                        self.last_mic = Some(*m);
                        return true;
                    }
                } else {
                    self.last_mic = Some(*m);
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

#[derive(Debug, Default)]
pub struct DutyCycler {
    current_interval_ms: u64,
    config: SamplingConfig,
}

impl DutyCycler {
    pub fn new(config: SamplingConfig) -> Self {
        Self { current_interval_ms: config.max_quiet_interval_ms, config }
    }
    pub fn update(&mut self, active: bool) {
        self.current_interval_ms = if active {
            self.config.active_interval_ms
        } else {
            self.config.max_quiet_interval_ms
        };
        self.current_interval_ms = self.current_interval_ms.max(self.config.min_interval_ms);
    }
    pub fn current_interval(&self) -> u64 {
        self.current_interval_ms
    }
}

#[derive(Debug)]
pub struct AcquisitionEngine {
    config: SamplingConfig,
    duty_cycler: DutyCycler,
    activity: ActivityDetector,
    event_bus: Arc<Mutex<Vec<SensorEvent>>>,
    sample_count: u64,
}

impl AcquisitionEngine {
    pub fn new(event_bus: Arc<Mutex<Vec<SensorEvent>>>) -> Self {
        Self {
            config: SamplingConfig::default(),
            duty_cycler: DutyCycler::new(SamplingConfig::default()),
            activity: ActivityDetector::new(),
            event_bus,
            sample_count: 0,
        }
    }

    fn now_ms() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::new(0, 0))
            .as_millis()
    }

    pub fn ingest(&mut self, sensor_type: SensorType, data: SensorData) -> SensorEvent {
        let event = SensorEvent {
            timestamp_ms: Self::now_ms(),
            sensor_type,
            data: data.clone(),
            confidence: confidence_for(sensor_type, &data),
        };
        let active = self.activity.is_active(&event, self.config.activity_threshold);
        self.duty_cycler.update(active);
        self.sample_count += 1;
        if let Ok(mut bus) = self.event_bus.lock() {
            bus.push(event.clone());
        }
        event
    }

    pub fn sampling_interval(&self) -> u64 {
        self.duty_cycler.current_interval()
    }
    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }
    pub fn is_active(&self) -> bool {
        self.duty_cycler.current_interval() == self.config.active_interval_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bus() -> Arc<Mutex<Vec<SensorEvent>>> {
        Arc::new(Mutex::new(Vec::new()))
    }

    #[test]
    fn first_sample_active_then_stillness_rests() {
        let mut e = AcquisitionEngine::new(bus());
        e.ingest(SensorType::Accelerometer, SensorData::accel(0.0, 0.0, 9.81));
        assert!(e.is_active());
        e.ingest(SensorType::Accelerometer, SensorData::accel(0.0, 0.0, 9.81));
        assert!(!e.is_active());
        assert_eq!(e.sample_count(), 2);
    }

    #[test]
    fn confidence_tracks_gps_quality() {
        let good = confidence_for(SensorType::Gps, &SensorData::gps(0.0, 0.0, 3.0));
        let bad = confidence_for(SensorType::Gps, &SensorData::gps(0.0, 0.0, 90.0));
        assert!(good > bad);
        assert!((0.0..=1.0).contains(&good));
    }

    #[test]
    fn events_land_on_bus_with_timestamps() {
        let b = bus();
        let mut e = AcquisitionEngine::new(Arc::clone(&b));
        e.ingest(SensorType::ScreenState, SensorData::screen(true, 80));
        let locked = b.lock().unwrap();
        assert_eq!(locked.len(), 1);
        assert!(locked[0].timestamp_ms > 0);
    }
}
