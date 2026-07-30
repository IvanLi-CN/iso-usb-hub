#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VoltageRange {
    InRange,
    Undervoltage,
    Overvoltage,
}

pub fn classify_voltage(vin_v: f32, min_v: f32, max_v: f32) -> VoltageRange {
    if (min_v..=max_v).contains(&vin_v) {
        VoltageRange::InRange
    } else if vin_v > max_v {
        VoltageRange::Overvoltage
    } else {
        VoltageRange::Undervoltage
    }
}

pub fn voltage_in_range(vin_v: f32, min_v: f32, max_v: f32) -> bool {
    classify_voltage(vin_v, min_v, max_v) == VoltageRange::InRange
}

#[cfg(test)]
mod tests {
    use super::{classify_voltage, voltage_in_range, VoltageRange};

    const VIN_MIN_V: f32 = 4.5;
    const VIN_MAX_V: f32 = 28.0;

    #[test]
    fn accepts_the_exact_28v_upper_limit() {
        assert_eq!(
            classify_voltage(28.0, VIN_MIN_V, VIN_MAX_V),
            VoltageRange::InRange
        );
        assert!(voltage_in_range(28.0, VIN_MIN_V, VIN_MAX_V));
    }

    #[test]
    fn rejects_voltage_above_the_28v_upper_limit() {
        assert_eq!(
            classify_voltage(28.001, VIN_MIN_V, VIN_MAX_V),
            VoltageRange::Overvoltage
        );
        assert!(!voltage_in_range(28.001, VIN_MIN_V, VIN_MAX_V));
    }

    #[test]
    fn preserves_undervoltage_rejection() {
        assert_eq!(
            classify_voltage(4.499, VIN_MIN_V, VIN_MAX_V),
            VoltageRange::Undervoltage
        );
        assert!(!voltage_in_range(4.499, VIN_MIN_V, VIN_MAX_V));
    }
}
