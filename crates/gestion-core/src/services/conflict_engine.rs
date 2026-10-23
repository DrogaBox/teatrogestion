use crate::domain::{DailyScheduleAggregate, PersonnelRole, RoomCode, WorkRegime};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "conflict_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScheduleConflict {
    /// Un bailarín, maestro o pianista figura en dos salas o actividades simultáneas
    PersonDoubleBooked {
        personnel_id: String,
        person_name: String,
        role: PersonnelRole,
        first_block_id: String,
        first_room: RoomCode,
        first_passage: String,
        second_block_id: String,
        second_room: RoomCode,
        second_passage: String,
        overlap_start: String,
        overlap_end: String,
    },
    /// Dos bloques distintos ocupan la misma sala en horarios superpuestos sin permiso de sala compartida
    RoomDoubleBooked {
        room: RoomCode,
        first_block_id: String,
        first_passage: String,
        second_block_id: String,
        second_passage: String,
        overlap_start: String,
        overlap_end: String,
    },
    /// Un ensayo excede las 13:45 h en jornada ordinaria o las 14:45 h en modo Pre-Función
    ShiftLimitExceeded {
        block_id: String,
        passage_name: String,
        block_end_time: String,
        current_regime: WorkRegime,
        allowed_limit_time: String,
        suggestion: String,
    },
    /// Descanso inter-jornada inferior al mínimo reglamentario (12 h) entre turno nocturno previo y jornada matutina
    MinimumIntershiftRestViolation {
        previous_date: String,
        previous_end_time: String,
        current_date: String,
        current_checkin_time: String,
        actual_rest_minutes: u32,
        required_rest_minutes: u32,
    },
    /// Descanso insuficiente entre la finalización de una clase técnica matutina y el inicio del primer bloque de ensayo
    ClassToRehearsalRestViolation {
        class_room: RoomCode,
        class_end_time: String,
        rehearsal_block_id: String,
        rehearsal_passage: String,
        rehearsal_start_time: String,
        actual_gap_minutes: u16,
        required_gap_minutes: u16,
    },
    /// Violación de franco compensatorio decretado por función previa
    CompensatoryRestDayViolation {
        date: String,
        rehearsal_blocks_count: usize,
        suggestion: String,
    },
}

/// Convierte una cadena `"HH:MM"` a minutos desde las `00:00`
pub fn parse_hhmm_to_minutes(hhmm: &str) -> Option<u16> {
    let (h_str, m_str) = hhmm.trim().split_once(':')?;
    let h: u16 = h_str.parse().ok()?;
    let m: u16 = m_str.parse().ok()?;
    if h <= 29 && m < 60 {
        Some(h * 60 + m)
    } else {
        None
    }
}

/// Formatea minutos desde las `00:00` en formato `"HH:MM"`
pub fn format_minutes_to_hhmm(mins: u16) -> String {
    format!("{:02}:{:02}", mins / 60, mins % 60)
}

#[derive(Clone)]
struct RoomIntervalEntry {
    block_id: String,
    room: RoomCode,
    passage_name: String,
    start_min: u16,
    end_min: u16,
    allow_shared_room: bool,
}

#[derive(Clone)]
struct PersonIntervalEntry {
    block_id: String,
    room: RoomCode,
    passage_label: String,
    start_min: u16,
    end_min: u16,
    person_id: String,
    person_name: String,
    role: PersonnelRole,
}

/// Audita en tiempo real (complejidad `O(N log N)` por barrido de intervalos ordenados) todas las reglas
/// operativas y laborales de un Parte Diario de Ensayo.
pub fn audit_daily_schedule(
    current: &DailyScheduleAggregate,
    previous_day_last_end_time: Option<(&str, &str)>,
) -> Vec<ScheduleConflict> {
    let mut conflicts = Vec::new();

    // 1. Validación de descanso mínimo inter-jornada (ej. 12 horas entre cierre nocturno D-1 y marcada 08:45 D)
    if let Some((prev_date, prev_end_hhmm)) = previous_day_last_end_time {
        if let (Some(prev_end_min), Some(curr_checkin_min)) = (
            parse_hhmm_to_minutes(prev_end_hhmm),
            parse_hhmm_to_minutes(&current.mandatory_checkin_time),
        ) {
            let elapsed_minutes =
                (24u32 * 60).saturating_sub(prev_end_min as u32) + (curr_checkin_min as u32);
            let required_minutes = (current.minimum_intershift_rest_hours as u32) * 60;

            if elapsed_minutes < required_minutes {
                conflicts.push(ScheduleConflict::MinimumIntershiftRestViolation {
                    previous_date: prev_date.to_string(),
                    previous_end_time: prev_end_hhmm.to_string(),
                    current_date: current.schedule_date.clone(),
                    current_checkin_time: current.mandatory_checkin_time.clone(),
                    actual_rest_minutes: elapsed_minutes,
                    required_rest_minutes: required_minutes,
                });
            }
        }
    }

    // 2. Validación de tope de jornada (13:45 h en ORDINARY, 14:45 h en PRE_FUNCTION)
    if let Some(limit_min) = current.regime.max_daytime_end_minutes() {
        for block in &current.rehearsal_blocks {
            if let Some(end_min) = parse_hhmm_to_minutes(&block.end_time) {
                if end_min > limit_min {
                    let suggestion = if current.regime == WorkRegime::Ordinary
                        && end_min <= WorkRegime::PRE_FUNCTION_END_LIMIT_MINUTES
                    {
                        "Activar modo PRE-FUNCIÓN (+1h máxima hasta las 14:45 h) o pasar a régimen EXTRAORDINARIO."
                            .to_string()
                    } else {
                        "Cambiar el Parte Diario a régimen EXTRAORDINARIO.".to_string()
                    };

                    conflicts.push(ScheduleConflict::ShiftLimitExceeded {
                        block_id: block.id.clone(),
                        passage_name: block.passage_name.clone(),
                        block_end_time: block.end_time.clone(),
                        current_regime: current.regime,
                        allowed_limit_time: format_minutes_to_hhmm(limit_min),
                        suggestion,
                    });
                }
            }
        }
    }

    // 2b. Validación de descanso reglamentario entre clase técnica y ensayos
    if current.class_to_rehearsal_rest_minutes > 0 && !current.morning_classes.is_empty() && !current.rehearsal_blocks.is_empty() {
        for mclass in &current.morning_classes {
            if let Some(class_end_min) = parse_hhmm_to_minutes(&mclass.end_time) {
                for rblock in &current.rehearsal_blocks {
                    if let Some(r_start_min) = parse_hhmm_to_minutes(&rblock.start_time) {
                        if r_start_min >= class_end_min && r_start_min < class_end_min + current.class_to_rehearsal_rest_minutes {
                            let gap = r_start_min.saturating_sub(class_end_min);
                            conflicts.push(ScheduleConflict::ClassToRehearsalRestViolation {
                                class_room: mclass.room,
                                class_end_time: mclass.end_time.clone(),
                                rehearsal_block_id: rblock.id.clone(),
                                rehearsal_passage: rblock.passage_name.clone(),
                                rehearsal_start_time: rblock.start_time.clone(),
                                actual_gap_minutes: gap,
                                required_gap_minutes: current.class_to_rehearsal_rest_minutes,
                            });
                        }
                    }
                }
            }
        }
    }

    // 2c. Validación de jornada declarada como Franco Compensatorio
    if current.is_compensatory_rest_day && !current.rehearsal_blocks.is_empty() {
        conflicts.push(ScheduleConflict::CompensatoryRestDayViolation {
            date: current.schedule_date.clone(),
            rehearsal_blocks_count: current.rehearsal_blocks.len(),
            suggestion: "Día declarado de Franco Compensatorio tras función. No deben citarse ensayos obligatorios sin acuerdo de Inspección.".to_string(),
        });
    }

    // 3. Detección O(N log N) de colisiones de Sala por barrido de intervalos ordenados
    let mut rooms_map: HashMap<RoomCode, Vec<RoomIntervalEntry>> = HashMap::new();
    for block in &current.rehearsal_blocks {
        if let (Some(start_min), Some(end_min)) = (
            parse_hhmm_to_minutes(&block.start_time),
            parse_hhmm_to_minutes(&block.end_time),
        ) {
            if start_min < end_min {
                rooms_map
                    .entry(block.room)
                    .or_default()
                    .push(RoomIntervalEntry {
                        block_id: block.id.clone(),
                        room: block.room,
                        passage_name: block.passage_name.clone(),
                        start_min,
                        end_min,
                        allow_shared_room: block.allow_shared_room,
                    });
            }
        }
    }

    let mut room_keys: Vec<RoomCode> = rooms_map.keys().copied().collect();
    room_keys.sort_by_key(|r| r.as_db_str());

    for room_code in room_keys {
        if let Some(mut intervals) = rooms_map.remove(&room_code) {
            intervals.sort_by(|a, b| {
                a.start_min
                    .cmp(&b.start_min)
                    .then_with(|| a.end_min.cmp(&b.end_min))
                    .then_with(|| a.block_id.cmp(&b.block_id))
            });

            for i in 0..intervals.len() {
                for j in (i + 1)..intervals.len() {
                    let first = &intervals[i];
                    let second = &intervals[j];
                    // Corte temprano del sweep-line: como están ordenados por start_min,
                    // si second.start_min >= first.end_min ningún intervalo posterior k > j colisionará con first.
                    if second.start_min >= first.end_min {
                        break;
                    }
                    if !(first.allow_shared_room && second.allow_shared_room) {
                        let overlap_start = first.start_min.max(second.start_min);
                        let overlap_end = first.end_min.min(second.end_min);
                        if overlap_start < overlap_end {
                            conflicts.push(ScheduleConflict::RoomDoubleBooked {
                                room: first.room,
                                first_block_id: first.block_id.clone(),
                                first_passage: first.passage_name.clone(),
                                second_block_id: second.block_id.clone(),
                                second_passage: second.passage_name.clone(),
                                overlap_start: format_minutes_to_hhmm(overlap_start),
                                overlap_end: format_minutes_to_hhmm(overlap_end),
                            });
                        }
                    }
                }
            }
        }
    }

    // 4. Recolección de intervalos de Personal (Maestros, Pianistas y Bailarines)
    let mut person_intervals: HashMap<String, Vec<PersonIntervalEntry>> = HashMap::new();

    for mc in &current.morning_classes {
        if let (Some(s), Some(e)) = (
            parse_hhmm_to_minutes(&mc.start_time),
            parse_hhmm_to_minutes(&mc.end_time),
        ) {
            person_intervals
                .entry(mc.master_id.clone())
                .or_default()
                .push(PersonIntervalEntry {
                    block_id: mc.id.clone(),
                    room: mc.room,
                    passage_label: "Clase Técnica Matutina".to_string(),
                    start_min: s,
                    end_min: e,
                    person_id: mc.master_id.clone(),
                    person_name: mc.master_name.clone(),
                    role: PersonnelRole::Master,
                });

            person_intervals
                .entry(mc.pianist_id.clone())
                .or_default()
                .push(PersonIntervalEntry {
                    block_id: mc.id.clone(),
                    room: mc.room,
                    passage_label: "Clase Técnica Matutina".to_string(),
                    start_min: s,
                    end_min: e,
                    person_id: mc.pianist_id.clone(),
                    person_name: mc.pianist_name.clone(),
                    role: PersonnelRole::Pianist,
                });

            if let (Some(t_id), Some(t_name)) = (&mc.sound_tech_id, &mc.sound_tech_name) {
                person_intervals
                    .entry(t_id.clone())
                    .or_default()
                    .push(PersonIntervalEntry {
                        block_id: mc.id.clone(),
                        room: mc.room,
                        passage_label: "Clase Técnica Matutina".to_string(),
                        start_min: s,
                        end_min: e,
                        person_id: t_id.clone(),
                        person_name: t_name.clone(),
                        role: PersonnelRole::SoundTech,
                    });
            }
        }
    }

    for block in &current.rehearsal_blocks {
        let (Some(s), Some(e)) = (
            parse_hhmm_to_minutes(&block.start_time),
            parse_hhmm_to_minutes(&block.end_time),
        ) else {
            continue;
        };

        if let (Some(m_id), Some(m_name)) = (&block.master_id, &block.master_name) {
            person_intervals
                .entry(m_id.clone())
                .or_default()
                .push(PersonIntervalEntry {
                    block_id: block.id.clone(),
                    room: block.room,
                    passage_label: block.passage_name.clone(),
                    start_min: s,
                    end_min: e,
                    person_id: m_id.clone(),
                    person_name: m_name.clone(),
                    role: PersonnelRole::Master,
                });
        }

        if let (Some(p_id), Some(p_name)) = (&block.pianist_id, &block.pianist_name) {
            person_intervals
                .entry(p_id.clone())
                .or_default()
                .push(PersonIntervalEntry {
                    block_id: block.id.clone(),
                    room: block.room,
                    passage_label: block.passage_name.clone(),
                    start_min: s,
                    end_min: e,
                    person_id: p_id.clone(),
                    person_name: p_name.clone(),
                    role: PersonnelRole::Pianist,
                });
        }

        if let (Some(t_id), Some(t_name)) = (&block.sound_tech_id, &block.sound_tech_name) {
            person_intervals
                .entry(t_id.clone())
                .or_default()
                .push(PersonIntervalEntry {
                    block_id: block.id.clone(),
                    room: block.room,
                    passage_label: block.passage_name.clone(),
                    start_min: s,
                    end_min: e,
                    person_id: t_id.clone(),
                    person_name: t_name.clone(),
                    role: PersonnelRole::SoundTech,
                });
        }

        let mut seen_dancers_in_block = HashSet::new();
        for group in &block.cast_groups {
            for member in &group.members {
                if seen_dancers_in_block.insert(member.personnel_id.clone()) {
                    person_intervals
                        .entry(member.personnel_id.clone())
                        .or_default()
                        .push(PersonIntervalEntry {
                            block_id: block.id.clone(),
                            room: block.room,
                            passage_label: block.passage_name.clone(),
                            start_min: s,
                            end_min: e,
                            person_id: member.personnel_id.clone(),
                            person_name: member.display_name.clone(),
                            role: PersonnelRole::Dancer,
                        });
                }
            }
        }
    }

    // 5. Detección O(N log N) de solapamiento de personas por barrido ordenado
    let mut person_ids: Vec<String> = person_intervals.keys().cloned().collect();
    person_ids.sort();

    for pid in person_ids {
        if let Some(mut list) = person_intervals.remove(&pid) {
            list.sort_by(|a, b| {
                a.start_min
                    .cmp(&b.start_min)
                    .then_with(|| a.end_min.cmp(&b.end_min))
                    .then_with(|| a.block_id.cmp(&b.block_id))
            });

            for i in 0..list.len() {
                for j in (i + 1)..list.len() {
                    let first = &list[i];
                    let second = &list[j];
                    if second.start_min >= first.end_min {
                        break;
                    }
                    let overlap_start = first.start_min.max(second.start_min);
                    let overlap_end = first.end_min.min(second.end_min);
                    if overlap_start < overlap_end && first.block_id != second.block_id {
                        conflicts.push(ScheduleConflict::PersonDoubleBooked {
                            personnel_id: first.person_id.clone(),
                            person_name: first.person_name.clone(),
                            role: first.role,
                            first_block_id: first.block_id.clone(),
                            first_room: first.room,
                            first_passage: first.passage_label.clone(),
                            second_block_id: second.block_id.clone(),
                            second_room: second.room,
                            second_passage: second.passage_label.clone(),
                            overlap_start: format_minutes_to_hhmm(overlap_start),
                            overlap_end: format_minutes_to_hhmm(overlap_end),
                        });
                    }
                }
            }
        }
    }

    conflicts
}
