// Extracted from xenofall.rs — Cards
// =============================================================================
// SISTEMA DE CARTAS — Roguelite modifiers
// =============================================================================

use crate::xenofall::constants::*;

#[derive(Clone, Copy, PartialEq)]
pub enum CardType {
    // Ofensivas
    DoubleTap,      // TAP damage x3 en vez de x2
    PiercingRounds, // Balas atraviesan 1 enemigo extra
    ExplosiveShot,  // Daño de área al impactar
    // Defensivas
    ArmorPlating, // +25 HP máximo
    Regeneration, // Regenerar 2 HP por oleada
    QuickReload,  // Tiempo de recarga -40%
    // Utilidad
    ExtendedMag, // +4 balas extra por cargador
    ComboMaster, // Combo timeout +2 seg
    ScoreBonus,  // +50% score por kill
}

impl CardType {
    pub fn name(self) -> &'static str {
        match self {
            Self::DoubleTap => "🔥 DOBLE TAP",
            Self::PiercingRounds => "🗡️ RONDAS PERFORANTES",
            Self::ExplosiveShot => "💥 DISPARO EXPLOSIVO",
            Self::ArmorPlating => "🛡️ BLINDAJE",
            Self::Regeneration => "💚 REGENERACIÓN",
            Self::QuickReload => "⚡ RECARGA RÁPIDA",
            Self::ExtendedMag => "📦 CARGADOR EXTENDIDO",
            Self::ComboMaster => "🎯 MAESTRO DEL COMBO",
            Self::ScoreBonus => "⭐ BONUS DE PUNTOS",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::DoubleTap => "TAP rápido → daño x3",
            Self::PiercingRounds => "Balas penetran +1 enemigo",
            Self::ExplosiveShot => "Daño en área al impactar",
            Self::ArmorPlating => "+25 HP máximo",
            Self::Regeneration => "+2 HP al inicio de oleada",
            Self::QuickReload => "Recarga 40% más rápida",
            Self::ExtendedMag => "+4 balas por cargador",
            Self::ComboMaster => "Combo timeout +2 segundos",
            Self::ScoreBonus => "+50% puntos por kill",
        }
    }

    /// Get all card types for random selection
    pub fn all() -> &'static [CardType] {
        &[
            Self::DoubleTap,
            Self::PiercingRounds,
            Self::ExplosiveShot,
            Self::ArmorPlating,
            Self::Regeneration,
            Self::QuickReload,
            Self::ExtendedMag,
            Self::ComboMaster,
            Self::ScoreBonus,
        ]
    }
}

/// Active build modifiers from selected cards
pub struct Build {
    pub tap_damage_mult: f32,
    pub piercing: u32,
    pub explosive_radius: f32,
    pub max_hp: i32,
    pub regen_per_wave: i32,
    pub reload_mult: f32,
    pub mag_bonus: u32,
    pub combo_timeout_bonus: f32,
    pub score_mult: f32,
    pub cards_collected: Vec<CardType>,
}

impl Build {
    pub fn new() -> Self {
        Self {
            tap_damage_mult: TAP_DAMAGE_MULT,
            piercing: 0,
            explosive_radius: 0.0,
            max_hp: PLAYER_MAX_HP,
            regen_per_wave: 0,
            reload_mult: 1.0,
            mag_bonus: 0,
            combo_timeout_bonus: 0.0,
            score_mult: 1.0,
            cards_collected: Vec::new(),
        }
    }

    pub fn apply_card(&mut self, card: CardType) {
        self.cards_collected.push(card);
        match card {
            CardType::DoubleTap => self.tap_damage_mult = 3.0,
            CardType::PiercingRounds => self.piercing += 1,
            CardType::ExplosiveShot => self.explosive_radius = 2.5,
            CardType::ArmorPlating => self.max_hp += 25,
            CardType::Regeneration => self.regen_per_wave += 2,
            CardType::QuickReload => self.reload_mult *= 0.6,
            CardType::ExtendedMag => self.mag_bonus += 4,
            CardType::ComboMaster => self.combo_timeout_bonus += 2.0,
            CardType::ScoreBonus => self.score_mult *= 1.5,
        }
    }

    pub fn effective_mag_size(&self) -> u32 {
        MAX_AMMO + self.mag_bonus
    }

    pub fn effective_reload_time(&self) -> f32 {
        RELOAD_TIME * self.reload_mult
    }

    pub fn effective_combo_timeout(&self) -> f32 {
        COMBO_TIMEOUT + self.combo_timeout_bonus
    }
}
