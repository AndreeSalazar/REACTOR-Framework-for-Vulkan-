pub fn generate_value_noise_3d(size: u32, seed: u32) -> Vec<u8> {
    let n = size as usize;
    let rng = seed.wrapping_mul(2_654_435_761u32);
    let mut data = vec![0u8; n * n * n * 4];
    for z in 0..n {
        for y in 0..n {
            for x in 0..n {
                let i = ((z * n * n + y * n + x) * 4) as usize;
                let mut h = rng
                    .wrapping_add((x as u32).wrapping_mul(73))
                    .wrapping_add((y as u32).wrapping_mul(91))
                    .wrapping_add((z as u32).wrapping_mul(127));
                h = h.wrapping_mul(1664525).wrapping_add(1013904223);
                data[i] = ((h >> 24) & 0xFF) as u8;
                data[i + 1] = ((h >> 16) & 0xFF) as u8;
                data[i + 2] = ((h >> 8) & 0xFF) as u8;
                data[i + 3] = 255;
            }
        }
    }
    let mut smooth = vec![0u8; data.len()];
    for z in 0..n {
        for y in 0..n {
            for x in 0..n {
                let xi = (x + 1) % n;
                let yi = (y + 1) % n;
                let zi = (z + 1) % n;
                let mut acc = [0u32; 4];
                for dz in 0..2 {
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let sx = if dx == 0 { x } else { xi };
                            let sy = if dy == 0 { y } else { yi };
                            let sz = if dz == 0 { z } else { zi };
                            let idx = ((sz * n * n + sy * n + sx) * 4) as usize;
                            acc[0] += data[idx] as u32;
                            acc[1] += data[idx + 1] as u32;
                            acc[2] += data[idx + 2] as u32;
                            acc[3] += data[idx + 3] as u32;
                        }
                    }
                }
                let o = ((z * n * n + y * n + x) * 4) as usize;
                smooth[o] = (acc[0] / 8) as u8;
                smooth[o + 1] = (acc[1] / 8) as u8;
                smooth[o + 2] = (acc[2] / 8) as u8;
                smooth[o + 3] = (acc[3] / 8) as u8;
            }
        }
    }
    smooth
}
