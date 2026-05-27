// =============================================================================
// generate_audio.rs — Procedural WAV Sound Effect Generator for Xenofall
// =============================================================================
// Generates 10 game-ready WAV files using pure Rust (std only).
// All files: PCM 16-bit, 44100 Hz, Mono.
//
// Usage:  cargo run --example generate_audio
// Output: assets/audio/*.wav
// =============================================================================

use std::f64::consts::PI;
use std::fs;
use std::io::Write;
use std::path::Path;

const SAMPLE_RATE: u32 = 44100;
const BITS_PER_SAMPLE: u16 = 16;
const NUM_CHANNELS: u16 = 1;

// ─── WAV Writer ──────────────────────────────────────────────────────────────

fn write_wav(path: &Path, samples: &[i16]) {
    let data_size = (samples.len() * 2) as u32;
    let file_size = 36 + data_size;

    let mut buf: Vec<u8> = Vec::with_capacity(file_size as usize + 8);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt  sub-chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // sub-chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    buf.extend_from_slice(&NUM_CHANNELS.to_le_bytes());
    buf.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    let byte_rate = SAMPLE_RATE * NUM_CHANNELS as u32 * BITS_PER_SAMPLE as u32 / 8;
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = NUM_CHANNELS * BITS_PER_SAMPLE / 8;
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());

    // data sub-chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());
    for &s in samples {
        buf.extend_from_slice(&s.to_le_bytes());
    }

    let mut file = fs::File::create(path).expect("Failed to create WAV file");
    file.write_all(&buf).expect("Failed to write WAV data");
    println!("  ✓ {}", path.display());
}

// ─── Utility Functions ───────────────────────────────────────────────────────

/// Simple pseudo-random number generator (xorshift64)
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xDEAD_BEEF_CAFE } else { seed },
        }
    }

    /// Returns a value in [-1.0, 1.0]
    fn next_f64(&mut self) -> f64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state as i64) as f64 / i64::MAX as f64
    }
}

fn seconds_to_samples(seconds: f64) -> usize {
    (SAMPLE_RATE as f64 * seconds) as usize
}

fn clamp_sample(v: f64) -> i16 {
    let v = v * 32767.0;
    if v > 32767.0 {
        32767
    } else if v < -32768.0 {
        -32768
    } else {
        v as i16
    }
}

fn sine(freq: f64, t: f64) -> f64 {
    (2.0 * PI * freq * t).sin()
}

fn time_at(i: usize) -> f64 {
    i as f64 / SAMPLE_RATE as f64
}

// ─── Sound Generators ────────────────────────────────────────────────────────

/// 1. gunshot.wav — Sharp gunshot
fn gen_gunshot() -> Vec<i16> {
    let total = seconds_to_samples(0.15);
    let mut rng = Rng::new(42);
    let mut samples = Vec::with_capacity(total);

    for i in 0..total {
        let t = time_at(i);

        // White noise burst with exponential decay (50ms main, tail after)
        let noise = rng.next_f64();
        let noise_env = (-t / 0.012).exp(); // fast exponential decay
        let noise_part = noise * noise_env * 0.8;

        // Low-frequency thump (80Hz sine, 30ms)
        let thump_env = if t < 0.03 { (-t / 0.015).exp() } else { 0.0 };
        let thump = sine(80.0, t) * thump_env * 0.7;

        // Extra punch: slight distortion on the attack
        let mut mixed = noise_part + thump;
        if t < 0.005 {
            mixed *= 1.5;
        }
        // Soft clip
        mixed = mixed.tanh();

        samples.push(clamp_sample(mixed));
    }
    samples
}

/// 2. reload.wav — Mechanical click + slide
fn gen_reload() -> Vec<i16> {
    let total = seconds_to_samples(0.55);
    let mut rng = Rng::new(123);
    let mut samples = Vec::with_capacity(total);

    let click1_start = 0.0;
    let click2_start = 0.203; // 200ms gap
    let slide_start = 0.22;

    for i in 0..total {
        let t = time_at(i);
        let mut val = 0.0;

        // Click 1: short metallic burst at ~4kHz
        if t >= click1_start && t < click1_start + 0.003 {
            let ct = t - click1_start;
            val += sine(4000.0, ct) * 0.8 * (1.0 - ct / 0.003);
            val += sine(6500.0, ct) * 0.3 * (1.0 - ct / 0.003);
        }

        // Click 2: slightly different tone
        if t >= click2_start && t < click2_start + 0.003 {
            let ct = t - click2_start;
            val += sine(3500.0, ct) * 0.7 * (1.0 - ct / 0.003);
            val += sine(7000.0, ct) * 0.35 * (1.0 - ct / 0.003);
        }

        // Slide sound: filtered noise sweep high→low over 300ms
        if t >= slide_start && t < slide_start + 0.3 {
            let st = t - slide_start;
            let progress = st / 0.3;
            let noise = rng.next_f64();
            // Simple low-pass simulation: mix noise with a sine that sweeps down
            let sweep_freq = 6000.0 * (1.0 - progress * 0.85);
            let filtered = noise * sine(sweep_freq, st).abs();
            let env = (1.0 - progress) * 0.3;
            val += filtered * env;
        } else {
            // Keep RNG advancing to avoid jumps
            let _ = rng.next_f64();
        }

        samples.push(clamp_sample(val));
    }
    samples
}

/// 3. zombie_groan.wav — Deep growl with FM and AM
fn gen_zombie_groan() -> Vec<i16> {
    let total = seconds_to_samples(0.8);
    let mut rng = Rng::new(666);
    let mut samples = Vec::with_capacity(total);

    for i in 0..total {
        let t = time_at(i);
        let progress = t / 0.8;

        // Base frequency modulates between 60-120Hz
        let freq_mod = sine(2.5, t) * 30.0; // slow wobble
        let base_freq = 90.0 + freq_mod;

        // FM synthesis: carrier + modulator for richer harmonic content
        let mod_signal = sine(1.7, t) * 20.0;
        let carrier = sine(base_freq + mod_signal, t);

        // Add a second harmonic layer
        let harmonic = sine(base_freq * 1.5 + mod_signal * 0.7, t) * 0.3;

        // Amplitude modulation (tremolo ~3Hz)
        let am = 0.7 + 0.3 * sine(3.0, t);

        // Envelope: fade in, sustain, fade out
        let env = if progress < 0.1 {
            progress / 0.1
        } else if progress > 0.75 {
            (1.0 - progress) / 0.25
        } else {
            1.0
        };

        // Add subtle noise for breathiness
        let breath = rng.next_f64() * 0.08;

        let val = (carrier + harmonic + breath) * am * env * 0.7;
        samples.push(clamp_sample(val));
    }
    samples
}

/// 4. impact.wav — Hit impact with mid-frequency resonance
fn gen_impact() -> Vec<i16> {
    let total = seconds_to_samples(0.1);
    let mut rng = Rng::new(99);
    let mut samples = Vec::with_capacity(total);

    for i in 0..total {
        let t = time_at(i);

        // Short noise burst (20ms) with fast decay
        let noise = rng.next_f64();
        let noise_env = if t < 0.02 { (-t / 0.008).exp() } else { 0.0 };
        let noise_part = noise * noise_env * 0.6;

        // Mid-frequency resonance (400Hz) with longer decay
        let res_env = (-t / 0.04).exp();
        let resonance = sine(400.0, t) * res_env * 0.7;

        // Sub-bass thump for weight
        let sub_env = (-t / 0.03).exp();
        let sub = sine(100.0, t) * sub_env * 0.4;

        let val = (noise_part + resonance + sub).tanh();
        samples.push(clamp_sample(val));
    }
    samples
}

/// 5. death.wav — Enemy death: descending pitch sweep + noise
fn gen_death() -> Vec<i16> {
    let total = seconds_to_samples(0.45);
    let mut rng = Rng::new(777);
    let mut samples = Vec::with_capacity(total);

    let mut phase = 0.0_f64;

    for i in 0..total {
        let t = time_at(i);
        let progress = t / 0.4;

        // Descending pitch: 800Hz → 100Hz (exponential curve)
        let freq = 800.0 * (0.125_f64).powf(progress.min(1.0)); // 800 * (100/800)^progress

        // Accumulate phase for smooth frequency sweep
        phase += 2.0 * PI * freq / SAMPLE_RATE as f64;
        let sweep = phase.sin();

        // Envelope
        let env = if progress < 1.0 {
            (-progress * 2.0).exp()
        } else {
            (-t / 0.05).exp() * 0.1
        };

        // Noise layer
        let noise = rng.next_f64() * 0.3 * env;

        let val = (sweep * env * 0.7 + noise).tanh();
        samples.push(clamp_sample(val));
    }
    samples
}

/// 6. combo.wav — Three ascending chime notes
fn gen_combo() -> Vec<i16> {
    let note_dur = 0.05;
    let gap = 0.015;
    let total_dur = 3.0 * note_dur + 2.0 * gap + 0.1; // plus tail
    let total = seconds_to_samples(total_dur);
    let mut samples = Vec::with_capacity(total);

    let notes = [800.0_f64, 1000.0, 1200.0];

    for i in 0..total {
        let t = time_at(i);
        let mut val = 0.0;

        for (idx, &freq) in notes.iter().enumerate() {
            let start = idx as f64 * (note_dur + gap);
            if t >= start {
                let nt = t - start;
                // Each note has attack and exponential decay
                let env = if nt < 0.005 {
                    nt / 0.005 // 5ms attack
                } else {
                    (-(nt - 0.005) / 0.04).exp()
                };

                // Pure tone + slight harmonic for chime character
                let tone =
                    sine(freq, nt) * 0.7 + sine(freq * 2.0, nt) * 0.2 + sine(freq * 3.0, nt) * 0.08;
                val += tone * env;
            }
        }

        val *= 0.5;
        samples.push(clamp_sample(val));
    }
    samples
}

/// 7. card_select.wav — Sparkle sound with tremolo and pitch rise
fn gen_card_select() -> Vec<i16> {
    let total = seconds_to_samples(0.3);
    let mut samples = Vec::with_capacity(total);

    for i in 0..total {
        let t = time_at(i);
        let progress = t / 0.3;

        // Base frequency with slight pitch rise
        let freq = 2000.0 + 400.0 * progress;

        // Main tone
        let tone = sine(freq, t);

        // Add shimmer harmonics
        let shimmer = sine(freq * 2.01, t) * 0.3 + sine(freq * 3.02, t) * 0.15;

        // Tremolo: amplitude modulation at 20Hz
        let tremolo = 0.6 + 0.4 * sine(20.0, t);

        // Envelope: quick attack, sustained, fade out
        let env = if progress < 0.05 {
            progress / 0.05
        } else {
            (-(progress - 0.05) / 0.4).exp()
        };

        let val = (tone + shimmer) * tremolo * env * 0.5;
        samples.push(clamp_sample(val));
    }
    samples
}

/// 8. damage.wav — Player hurt: low thud + high crack
fn gen_damage() -> Vec<i16> {
    let total = seconds_to_samples(0.15);
    let mut rng = Rng::new(321);
    let mut samples = Vec::with_capacity(total);

    for i in 0..total {
        let t = time_at(i);

        // Low distorted thud (100Hz)
        let thud_env = (-t / 0.06).exp();
        let thud = sine(100.0, t) * thud_env;
        // Add distortion harmonics
        let thud_dist = (thud * 3.0).tanh() * 0.6;

        // High crack: 1500Hz burst, 10ms
        let crack_env = if t < 0.01 {
            (-t / 0.004).exp()
        } else {
            (-(t - 0.01) / 0.01).exp() * 0.2
        };
        let crack = sine(1500.0, t) * crack_env * 0.5;
        let crack_noise = rng.next_f64() * crack_env * 0.3;

        let val = (thud_dist + crack + crack_noise).tanh();
        samples.push(clamp_sample(val));
    }
    samples
}

/// 9. wave_start.wav — Alarm-like ascending tone, two pulses
fn gen_wave_start() -> Vec<i16> {
    let total = seconds_to_samples(0.5);
    let mut samples = Vec::with_capacity(total);

    for i in 0..total {
        let t = time_at(i);

        // Two pulses: 0-0.22s and 0.25-0.47s
        let (in_pulse, pulse_t) = if t < 0.22 {
            (true, t)
        } else if t >= 0.25 && t < 0.47 {
            (true, t - 0.25)
        } else {
            (false, 0.0)
        };

        if in_pulse {
            let pulse_dur = 0.22;
            let progress = pulse_t / pulse_dur;

            // Ascending tone: 400Hz → 800Hz
            let freq = 400.0 + 400.0 * progress;
            let tone = sine(freq, pulse_t);

            // Add slight harmonic edge
            let edge = sine(freq * 2.0, pulse_t) * 0.2;

            // Envelope per pulse
            let env = if progress < 0.05 {
                progress / 0.05
            } else if progress > 0.85 {
                (1.0 - progress) / 0.15
            } else {
                1.0
            };

            let val = (tone + edge) * env * 0.6;
            samples.push(clamp_sample(val));
        } else {
            samples.push(0);
        }
    }
    samples
}

/// 10. victory.wav — Ascending chord (C5-E5-G5) with chorus effect
fn gen_victory() -> Vec<i16> {
    let total = seconds_to_samples(1.0);
    let mut samples = Vec::with_capacity(total);

    let freqs = [523.25_f64, 659.25, 783.99]; // C5, E5, G5

    for i in 0..total {
        let t = time_at(i);
        let progress = t / 1.0;

        // Envelope: fade in 100ms, sustain, fade out last 200ms
        let env = if progress < 0.1 {
            progress / 0.1
        } else if progress > 0.8 {
            (1.0 - progress) / 0.2
        } else {
            1.0
        };

        let mut val = 0.0;
        for &freq in &freqs {
            // Main tone
            let main = sine(freq, t);

            // Chorus: slightly detuned copies
            let chorus1 = sine(freq * 1.003, t + 0.001);
            let chorus2 = sine(freq * 0.997, t + 0.002);

            // Combine with chorus effect
            val += main * 0.5 + chorus1 * 0.25 + chorus2 * 0.25;
        }

        // Normalize for 3 notes
        val /= 3.0;

        // Add warmth: slight even harmonic
        let warmth = sine(freqs[0] * 0.5, t) * 0.1; // sub-octave of root

        val = (val + warmth) * env * 0.7;
        samples.push(clamp_sample(val));
    }
    samples
}

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() {
    let audio_dir = Path::new("assets/audio");

    // Create output directory
    fs::create_dir_all(audio_dir).expect("Failed to create assets/audio directory");

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║   XENOFALL — Procedural Audio Generator                 ║");
    println!("║   PCM 16-bit · 44100 Hz · Mono                         ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let sound_effects: Vec<(&str, fn() -> Vec<i16>)> = vec![
        ("gunshot.wav", gen_gunshot),
        ("reload.wav", gen_reload),
        ("zombie_groan.wav", gen_zombie_groan),
        ("impact.wav", gen_impact),
        ("death.wav", gen_death),
        ("combo.wav", gen_combo),
        ("card_select.wav", gen_card_select),
        ("damage.wav", gen_damage),
        ("wave_start.wav", gen_wave_start),
        ("victory.wav", gen_victory),
    ];

    let mut total_bytes: u64 = 0;

    for (name, generator) in &sound_effects {
        let samples = generator();
        let path = audio_dir.join(name);
        let file_size = 44 + samples.len() * 2; // header + data
        total_bytes += file_size as u64;
        write_wav(&path, &samples);
    }

    println!();
    println!(
        "  Generated {} files ({:.1} KB total)",
        sound_effects.len(),
        total_bytes as f64 / 1024.0
    );
    println!("  Output: {}", audio_dir.display());
    println!();
    println!("  🎮 Ready for Xenofall!");
}
