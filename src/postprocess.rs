use crate::math::Vec3;

/// Separable nine-tap bloom with sliding sums: identical kernel, less repeated work.
pub fn finish(
    hdr: &[Vec3],
    scratch: &mut [Vec3],
    bloom: &mut [Vec3],
    pixels: &mut [u32],
    width: usize,
    height: usize,
) {
    for (source, dest) in hdr.iter().zip(bloom.iter_mut()) {
        let peak = source.x.max(source.y).max(source.z);
        *dest = if peak > 1.1 {
            *source * ((peak - 1.1) / peak)
        } else {
            Vec3::default()
        };
    }
    for y in 0..height {
        let row = &bloom[y * width..(y + 1) * width];
        let mut sum = row[0] * 4.0;
        for x in 0..=4 {
            sum += row[x.min(width - 1)];
        }
        for x in 0..width {
            scratch[y * width + x] = sum / 9.0;
            sum = sum - row[x.saturating_sub(4)] + row[(x + 5).min(width - 1)];
        }
    }
    // Reuse bloom as the vertical result, processing whole rows contiguously.
    for x in 0..width {
        let mut sum = scratch[x] * 4.0;
        for y in 0..=4 {
            sum += scratch[y.min(height - 1) * width + x];
        }
        for y in 0..height {
            bloom[y * width + x] = sum * (0.20 / 9.0);
            sum = sum - scratch[y.saturating_sub(4) * width + x]
                + scratch[(y + 5).min(height - 1) * width + x];
        }
    }
    for ((source, glow), dest) in hdr.iter().zip(bloom.iter()).zip(pixels.iter_mut()) {
        let color = *source + *glow;
        let channel = |v: f32| {
            let v = v.max(0.0);
            ((v / (v + 0.7)).sqrt() * 255.0 + 0.5) as u32
        };
        *dest = (channel(color.x) << 16) | (channel(color.y) << 8) | channel(color.z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sliding_bloom_matches_original_kernel() {
        for (width, height) in [(1, 1), (3, 2), (31, 17)] {
            let hdr: Vec<_> = (0..width * height)
                .map(|i| {
                    Vec3::new(
                        (i % 13) as f32 * 0.7,
                        (i % 7) as f32 * 0.4,
                        (i % 19) as f32 * 0.3,
                    )
                })
                .collect();
            let mut a = vec![0; width * height];
            let mut b = a.clone();
            let mut scratch = vec![Vec3::default(); a.len()];
            let mut bloom = scratch.clone();
            finish(&hdr, &mut scratch, &mut bloom, &mut a, width, height);
            reference(&hdr, &mut scratch, &mut bloom, &mut b, width, height);
            for (a, b) in a.into_iter().zip(b) {
                for shift in [0, 8, 16] {
                    assert!((((a >> shift) & 255) as i32 - ((b >> shift) & 255) as i32).abs() <= 1);
                }
            }
        }
    }

    /// Small separable bloom, followed by tone mapping. No GPU or shader dependency.
    fn reference(
        hdr: &[Vec3],
        scratch: &mut [Vec3],
        bloom: &mut [Vec3],
        pixels: &mut [u32],
        width: usize,
        height: usize,
    ) {
        for (source, dest) in hdr.iter().zip(bloom.iter_mut()) {
            let peak = source.x.max(source.y).max(source.z);
            *dest = if peak > 1.1 {
                *source * ((peak - 1.1) / peak)
            } else {
                Vec3::default()
            };
        }
        for y in 0..height {
            for x in 0..width {
                let mut sum = Vec3::default();
                for dx in -4isize..=4 {
                    sum += bloom[y * width + x.saturating_add_signed(dx).min(width - 1)];
                }
                scratch[y * width + x] = sum / 9.0;
            }
        }
        for y in 0..height {
            for x in 0..width {
                let mut sum = Vec3::default();
                for dy in -4isize..=4 {
                    sum += scratch[y.saturating_add_signed(dy).min(height - 1) * width + x];
                }
                let color = hdr[y * width + x] + sum * (0.20 / 9.0);
                let channel = |v: f32| {
                    let v = v.max(0.0);
                    ((v / (v + 0.7)).sqrt() * 255.0 + 0.5) as u32
                };
                pixels[y * width + x] =
                    (channel(color.x) << 16) | (channel(color.y) << 8) | channel(color.z);
            }
        }
    }
}
