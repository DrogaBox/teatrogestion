use crate::domain::{
    ActivityType, CastAssignmentCondition, CastMemberEntry, ConfirmationState,
    DailyScheduleAggregate, GenderGroup, MorningClassSlot, PersonnelRole, RehearsalBlock,
    RehearsalCastGroup, RoomCode, WorkRegime,
};
use crate::services::conflict_engine::{audit_daily_schedule, ScheduleConflict};

fn sample_cast_member(id: &str, pid: &str, name: &str, role: &str) -> CastMemberEntry {
    CastMemberEntry {
        id: id.to_string(),
        personnel_id: pid.to_string(),
        display_name: name.to_string(),
        character_role: Some(role.to_string()),
        partner_personnel_id: None,
        partner_display_name: None,
        condition: CastAssignmentCondition::Titular,
    }
}

fn sample_rehearsal_block(
    id: &str,
    room: RoomCode,
    start_time: &str,
    end_time: &str,
    passage_name: &str,
    master: Option<(&str, &str)>,
    pianist: Option<(&str, &str)>,
    members: Vec<CastMemberEntry>,
) -> RehearsalBlock {
    RehearsalBlock {
        id: id.to_string(),
        schedule_id: "SCH-2026-10-21".to_string(),
        room,
        start_time: start_time.to_string(),
        end_time: end_time.to_string(),
        activity_type: ActivityType::StudioRehearsal,
        work_id: Some("WORK_LAGO_CISNES".to_string()),
        work_title: Some("El Lago de los Cisnes".to_string()),
        passage_id: None,
        passage_name: passage_name.to_string(),
        confirmation_state: ConfirmationState::Confirmed,
        sound_requirement: "PISTA TeatroCue #08".to_string(),
        teatrocue_cue_ref: Some("CUE-08".to_string()),
        master_id: master.map(|(id, _)| id.to_string()),
        master_name: master.map(|(_, name)| name.to_string()),
        pianist_id: pianist.map(|(id, _)| id.to_string()),
        pianist_name: pianist.map(|(_, name)| name.to_string()),
        allow_shared_room: false,
        cast_groups: vec![RehearsalCastGroup {
            id: format!("GRP-{id}"),
            group_order: 1,
            group_label: Some("1er Reparto".to_string()),
            formatted_line: "Álvarez/Marcato - Morchio C/Gil".to_string(),
            is_to_confirm: false,
            members,
        }],
    }
}

#[test]
fn test_contiguous_blocks_have_no_false_positive_collisions() {
    let mut schedule = DailyScheduleAggregate::new_default(
        "SCH-2026-10-21".to_string(),
        "2026-10-21".to_string(),
        WorkRegime::Ordinary,
    );

    schedule.morning_classes = vec![
        MorningClassSlot {
            id: "MC-SALA1".to_string(),
            room: RoomCode::Sala1,
            gender_division: GenderGroup::Female,
            start_time: "09:00".to_string(),
            end_time: "10:15".to_string(),
            master_id: "PER_MASTER_PINO".to_string(),
            master_name: "Pino".to_string(),
            pianist_id: "PER_PIANIST_ROMERO".to_string(),
            pianist_name: "Romero".to_string(),
        },
        MorningClassSlot {
            id: "MC-SALA3".to_string(),
            room: RoomCode::Sala3,
            gender_division: GenderGroup::Male,
            start_time: "09:00".to_string(),
            end_time: "10:15".to_string(),
            master_id: "PER_MASTER_ANANIEV".to_string(),
            master_name: "Ananiev".to_string(),
            pianist_id: "PER_PIANIST_VILLEGAS".to_string(),
            pianist_name: "Villegas".to_string(),
        },
    ];

    // Bloques contiguos exactos: 10:30-11:30, 11:30-12:30, 12:30-13:45 en Sala 1 con mismos bailarines
    schedule.rehearsal_blocks = vec![
        sample_rehearsal_block(
            "BLK-1",
            RoomCode::Sala1,
            "10:30",
            "11:30",
            "Pas de deux Odette y Sigfrido",
            Some(("PER_MASTER_PINO", "Pino")),
            Some(("PER_PIANIST_ROMERO", "Romero")),
            vec![
                sample_cast_member("M1", "PER_DANCER_ALVAREZ", "Álvarez", "Odette"),
                sample_cast_member("M2", "PER_DANCER_MORCHIO_C", "Morchio C.", "Sigfrido"),
            ],
        ),
        sample_rehearsal_block(
            "BLK-2",
            RoomCode::Sala1,
            "11:30",
            "12:30",
            "Cisne Negro",
            Some(("PER_MASTER_PINO", "Pino")),
            Some(("PER_PIANIST_ROMERO", "Romero")),
            vec![
                sample_cast_member("M3", "PER_DANCER_ALVAREZ", "Álvarez", "Odile"),
                sample_cast_member("M4", "PER_DANCER_MORCHIO_C", "Morchio C.", "Sigfrido"),
            ],
        ),
        sample_rehearsal_block(
            "BLK-3",
            RoomCode::Sala1,
            "12:30",
            "13:45",
            "Princesas y Bufón",
            Some(("PER_MASTER_BENITES", "Benites")),
            Some(("PER_PIANIST_BRUSUTTI", "Brusutti")),
            vec![sample_cast_member("M5", "PER_DANCER_GIL", "Gil", "Bufón")],
        ),
    ];

    let conflicts = audit_daily_schedule(&schedule, Some(("2026-10-20", "20:00")));
    assert!(
        conflicts.is_empty(),
        "No debe haber conflictos en bloques contiguos reglamentarios: {:?}",
        conflicts
    );
}

#[test]
fn test_detects_dancer_and_pianist_cross_room_collision() {
    let mut schedule = DailyScheduleAggregate::new_default(
        "SCH-2026-10-21".to_string(),
        "2026-10-21".to_string(),
        WorkRegime::Ordinary,
    );

    // Sala 1: 10:30 a 11:30 (Álvarez + Pianista Romero)
    // Sala 3: 11:00 a 12:00 (Álvarez + Pianista Romero) -> Solapamiento 11:00 a 11:30
    schedule.rehearsal_blocks = vec![
        sample_rehearsal_block(
            "BLK-S1",
            RoomCode::Sala1,
            "10:30",
            "11:30",
            "Pas de deux Odette y Sigfrido",
            Some(("PER_MASTER_PINO", "Pino")),
            Some(("PER_PIANIST_ROMERO", "Romero")),
            vec![sample_cast_member("M1", "PER_DANCER_ALVAREZ", "Álvarez", "Odette")],
        ),
        sample_rehearsal_block(
            "BLK-S3",
            RoomCode::Sala3,
            "11:00",
            "12:00",
            "Diana y Acteón",
            Some(("PER_MASTER_ANANIEV", "Ananiev")),
            Some(("PER_PIANIST_ROMERO", "Romero")),
            vec![sample_cast_member("M2", "PER_DANCER_ALVAREZ", "Álvarez", "Diana")],
        ),
    ];

    let conflicts = audit_daily_schedule(&schedule, None);
    assert_eq!(conflicts.len(), 2);

    let has_dancer_conflict = conflicts.iter().any(|c| {
        matches!(
            c,
            ScheduleConflict::PersonDoubleBooked {
                personnel_id,
                role: PersonnelRole::Dancer,
                overlap_start,
                overlap_end,
                ..
            } if personnel_id == "PER_DANCER_ALVAREZ" && overlap_start == "11:00" && overlap_end == "11:30"
        )
    });
    assert!(has_dancer_conflict, "Debe detectar cruce de bailarina Álvarez");

    let has_pianist_conflict = conflicts.iter().any(|c| {
        matches!(
            c,
            ScheduleConflict::PersonDoubleBooked {
                personnel_id,
                role: PersonnelRole::Pianist,
                overlap_start,
                overlap_end,
                ..
            } if personnel_id == "PER_PIANIST_ROMERO" && overlap_start == "11:00" && overlap_end == "11:30"
        )
    });
    assert!(has_pianist_conflict, "Debe detectar cruce del pianista Romero");
}

#[test]
fn test_detects_room_double_booking() {
    let mut schedule = DailyScheduleAggregate::new_default(
        "SCH-2026-10-21".to_string(),
        "2026-10-21".to_string(),
        WorkRegime::Ordinary,
    );

    schedule.rehearsal_blocks = vec![
        sample_rehearsal_block(
            "BLK-A",
            RoomCode::Sala3,
            "11:30",
            "12:30",
            "Danza Española",
            Some(("PER_MASTER_BENITES", "Benites")),
            None,
            vec![],
        ),
        sample_rehearsal_block(
            "BLK-B",
            RoomCode::Sala3,
            "12:00",
            "13:00",
            "Princesas y Bufón",
            Some(("PER_MASTER_PINO", "Pino")),
            None,
            vec![],
        ),
    ];

    let conflicts = audit_daily_schedule(&schedule, None);
    assert_eq!(
        conflicts,
        vec![ScheduleConflict::RoomDoubleBooked {
            room: RoomCode::Sala3,
            first_block_id: "BLK-A".to_string(),
            first_passage: "Danza Española".to_string(),
            second_block_id: "BLK-B".to_string(),
            second_passage: "Princesas y Bufón".to_string(),
            overlap_start: "12:00".to_string(),
            overlap_end: "12:30".to_string(),
        }]
    );
}

#[test]
fn test_shift_limit_1345_ordinary_vs_1445_prefunction() {
    let mut schedule = DailyScheduleAggregate::new_default(
        "SCH-2026-10-21".to_string(),
        "2026-10-21".to_string(),
        WorkRegime::Ordinary,
    );

    schedule.rehearsal_blocks = vec![sample_rehearsal_block(
        "BLK-LATE",
        RoomCode::Escenario,
        "12:30",
        "14:30",
        "Pas de deux Odette y Sigfrido",
        None,
        None,
        vec![],
    )];

    // En régimen ORDINARY (tope 13:45), un ensayo hasta las 14:30 dispara ShiftLimitExceeded
    let conflicts_ord = audit_daily_schedule(&schedule, None);
    assert_eq!(conflicts_ord.len(), 1);
    assert!(matches!(
        &conflicts_ord[0],
        ScheduleConflict::ShiftLimitExceeded {
            allowed_limit_time,
            current_regime: WorkRegime::Ordinary,
            ..
        } if allowed_limit_time == "13:45"
    ));

    // Al activar modo PRE-FUNCIÓN (tope +1h hasta 14:45), el ensayo de 14:30 pasa a ser válido
    schedule.regime = WorkRegime::PreFunction;
    let conflicts_pre = audit_daily_schedule(&schedule, None);
    assert!(conflicts_pre.is_empty());
}

#[test]
fn test_minimum_intershift_rest_12h_validation() {
    let schedule = DailyScheduleAggregate::new_default(
        "SCH-2026-10-21".to_string(),
        "2026-10-21".to_string(),
        WorkRegime::Ordinary,
    );

    // Función nocturna el día anterior terminó a las 22:30 -> hasta las 08:45 hay solo 10h 15m (615 min < 720 min)
    let conflicts = audit_daily_schedule(&schedule, Some(("2026-10-20", "22:30")));
    assert_eq!(
        conflicts,
        vec![ScheduleConflict::MinimumIntershiftRestViolation {
            previous_date: "2026-10-20".to_string(),
            previous_end_time: "22:30".to_string(),
            current_date: "2026-10-21".to_string(),
            current_checkin_time: "08:45".to_string(),
            actual_rest_minutes: 615,
            required_rest_minutes: 720,
        }]
    );
}

#[test]
fn test_class_to_rehearsal_rest_violation() {
    let mut schedule = DailyScheduleAggregate::new_default(
        "SCH-2026-10-21".to_string(),
        "2026-10-21".to_string(),
        WorkRegime::Ordinary,
    );
    schedule.class_to_rehearsal_rest_minutes = 15;
    schedule.morning_classes = vec![MorningClassSlot {
        id: "MC-1".to_string(),
        room: RoomCode::Sala1,
        gender_division: GenderGroup::Female,
        start_time: "09:00".to_string(),
        end_time: "10:15".to_string(),
        master_id: "PER_MASTER_PINO".to_string(),
        master_name: "Pino".to_string(),
        pianist_id: "PER_PIANIST_ROMERO".to_string(),
        pianist_name: "Romero".to_string(),
    }];

    // Ensayo comienza a las 10:20 (solo 5 min de descanso tras terminar la clase a las 10:15)
    schedule.rehearsal_blocks = vec![sample_rehearsal_block(
        "BLK-1",
        RoomCode::Sala1,
        "10:20",
        "11:30",
        "Pas de deux",
        None,
        None,
        vec![],
    )];

    let conflicts = audit_daily_schedule(&schedule, None);
    assert_eq!(conflicts.len(), 1);
    assert!(matches!(
        &conflicts[0],
        ScheduleConflict::ClassToRehearsalRestViolation {
            actual_gap_minutes: 5,
            required_gap_minutes: 15,
            ..
        }
    ));

    // Si el ensayo comienza a las 10:30 (15 min de descanso exactos), no hay conflicto
    schedule.rehearsal_blocks[0].start_time = "10:30".to_string();
    let conflicts_ok = audit_daily_schedule(&schedule, None);
    assert!(conflicts_ok.is_empty());
}

#[test]
fn test_compensatory_rest_day_violation() {
    let mut schedule = DailyScheduleAggregate::new_default(
        "SCH-2026-10-21".to_string(),
        "2026-10-21".to_string(),
        WorkRegime::Ordinary,
    );
    schedule.is_compensatory_rest_day = true;
    schedule.compensatory_rest_policy = "NEXT_DAY".to_string();

    // Sin ensayos citados en día de franco: OK
    schedule.rehearsal_blocks = vec![];
    assert!(audit_daily_schedule(&schedule, None).is_empty());

    // Si se agrega un ensayo obligatorio en día compensatorio: detecta violación
    schedule.rehearsal_blocks = vec![sample_rehearsal_block(
        "BLK-1",
        RoomCode::Sala1,
        "10:30",
        "12:00",
        "Ensayo de reposición",
        None,
        None,
        vec![],
    )];

    let conflicts = audit_daily_schedule(&schedule, None);
    assert_eq!(conflicts.len(), 1);
    assert!(matches!(
        &conflicts[0],
        ScheduleConflict::CompensatoryRestDayViolation {
            rehearsal_blocks_count: 1,
            ..
        }
    ));
}

