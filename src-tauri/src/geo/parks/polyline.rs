//! Decodes Google polyline strings, as `scripts/geodata/park_lines.py` writes them.

const SCALE: f64 = 1e5;

/// The `(latitude, longitude)` points of one encoded ring.
pub(super) fn decode(encoded: &str) -> Option<Vec<(f32, f32)>> {
    let mut bytes = encoded.bytes();
    let mut points = Vec::with_capacity(encoded.len() / 4);
    let (mut latitude, mut longitude) = (0i64, 0i64);
    while bytes.len() > 0 {
        latitude += next_number(&mut bytes)?;
        longitude += next_number(&mut bytes)?;
        points.push((
            (latitude as f64 / SCALE) as f32,
            (longitude as f64 / SCALE) as f32,
        ));
    }
    Some(points)
}

fn next_number(bytes: &mut impl Iterator<Item = u8>) -> Option<i64> {
    let (mut value, mut shift) = (0i64, 0);
    loop {
        let chunk = i64::from(bytes.next()?.checked_sub(63)?);
        value |= (chunk & 0x1F) << shift;
        shift += 5;
        if chunk < 0x20 {
            break;
        }
    }
    Some(if value & 1 == 1 {
        !(value >> 1)
    } else {
        value >> 1
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn decodes_the_reference_example() {
        let points = super::decode("_p~iF~ps|U_ulLnnqC_mqNvxq`@").expect("valid");
        assert_eq!(
            points,
            [(38.5, -120.2), (40.7, -120.95), (43.252, -126.453)]
        );
    }
}
