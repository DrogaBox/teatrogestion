// Arnés de revisión visual en navegador (NO entra en la app real): datos de ejemplo.
const names = ['Altamirano, Ignacio','Beacon, Federico','Bravo, Silvia','Cornejo, Facundo Miguel','Cornejo, Matías','Gacitúa, Rodrigo','Gil, Nicolás','González, Martina','González, Sofía','Gómez, Sofía','Hazel, Ezequiel','Marcato, Lucía','Marenchino, María Belén','Martini, Valentín','Mauro, Agustín'];
const codes = ['P','P','P','EI','P','SP','IJ','P','LC','P'];
function matrix(year: number, month: number) {
  const dim = new Date(year, month, 0).getDate();
  const dn = ['D','L','M','M','J','V','S'];
  const days_header = Array.from({ length: dim }, (_, i) => {
    const d = i + 1, wd = new Date(year, month - 1, d).getDay();
    return { day_number: d, date: `${year}-${String(month).padStart(2,'0')}-${String(d).padStart(2,'0')}`, day_name: dn[wd], is_weekend: wd === 0 || wd === 6, day_annotation: d === 11 ? 'ENSAYO SUSPENDIDO POR LLUVIA' : null };
  });
  const rows = names.map((n, k) => {
    const cells = days_header.map(h => ({ day_number: h.day_number, date: h.date, is_weekend: h.is_weekend, code: h.is_weekend ? '' : codes[(h.day_number + k) % codes.length], minutes_late: 0, remarks: null }));
    return { nro: k + 1, personnel_id: 'p' + k, dancer_name: n, legajo: String(1000 + k), role: 'DANCER', cells,
      total_p: cells.filter(c => c.code === 'P').length, total_ei: cells.filter(c => c.code === 'EI').length, total_sp: cells.filter(c => c.code === 'SP').length,
      total_other: cells.filter(c => c.code && !['P','EI','SP','S/P'].includes(c.code)).length };
  });
  return { year, month, month_name: 'Septiembre', days_in_month: dim, days_header, rows };
}
const schedule = {
  id: 'S1', schedule_date: new Date().toISOString().slice(0,10), regime: 'ORDINARY', mandatory_checkin_time: '08:45', nominal_end_time: '13:45',
  minimum_intershift_rest_hours: 12, class_to_rehearsal_rest_minutes: 15, is_compensatory_rest_day: false, compensatory_rest_policy: 'x', footer_legal_clause: 'REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS.',
  morning_classes: [{ id: 'm1', room: 'SALA_1', gender_division: 'FEMALE', start_time: '09:00', end_time: '10:15', master_id: 'a', master_name: 'Prof. Rivas', pianist_id: 'b', pianist_name: 'Pianista Luna' }],
  rehearsal_blocks: [{ id: 'r1', schedule_id: 'S1', room: 'SALA_2', start_time: '10:30', end_time: '13:45', activity_type: 'STUDIO_REHEARSAL', work_id: null, work_title: 'Giselle', passage_id: null, passage_name: 'Acto II – Willis', confirmation_state: 'CONFIRMED', sound_requirement: '', teatrocue_cue_ref: null, master_id: null, master_name: 'Prof. Rivas', pianist_id: null, pianist_name: null, allow_shared_room: false, cast_groups: [] }],
};
const att = names.map((n, k) => ({ id: 'a' + k, schedule_id: 'S1', schedule_date: schedule.schedule_date, personnel_id: 'p' + k, dancer_name: n, legajo: String(1000 + k), role: 'DANCER', category_name: 'Cuerpo de baile', expected_checkin_time: '08:45', actual_checkin_time: null, attendance_status: k % 5 === 0 ? 'LATE' : 'PRESENT', is_in_class: true, minutes_late: 0, remarks: null, certificate_file_name: null, certificate_data: null, medical_diagnosis: null, doctor_details: null, leave_end_date: null, physio_notes: null, current_physio_diagnosis: null, physio_tracking_notes: null, recorded_at: '' }));
const pers = names.map((n, k) => { const [l, f] = n.split(', '); return { id: 'p' + k, personnel_id: 'p' + k, legajo: String(1000 + k), first_name: f, last_name: l, nickname: null, role: 'DANCER', gender_group: k % 2 ? 'MALE' : 'FEMALE', is_active: true, category_name: 'Cuerpo de baile', display_name: n }; });
const stock = Array.from({ length: 6 }, (_, k) => ({ id: 's' + k, sku_code: 'PUN-' + k, family: 'CALZADO', subcategory: 'PUNTA_GAYNOR_MINDEN', brand: 'GaynorMinden', model_name: 'Europeo Pro ' + k, raw_factory_code: null, color: 'Rosa', material: 'Satén', prop_dye_state: 'ORIGINAL', size_system: 'EU', size_value: String(36 + k), width_value: 'M', gm_line: null, gm_fit: null, gm_box: null, gm_shank: null, gm_vamp: null, gm_heel: null, duval_hardness: null, physical_qty: 5 - k % 4, reserved_qty: 1, free_available_qty: 4 - k % 4, storage_bin_location: 'A-' + k, is_low_stock: k % 3 === 0 }));
const works = [{ id: 'w1', title: 'Giselle', composer: 'Adolphe Adam', choreographer: 'Coralli / Perrot', is_active: true,
  passages: [{ id: 'ps1', work_id: 'w1', name: 'Acto II – Pas de deux', act_scene: 'II', default_duration_min: 25, default_sound_cue: 'GIS-201' }],
  casting_plans: [{ id: 'cp1', work_id: 'w1', work_title: 'Giselle', passage_id: 'ps1', passage_name: 'Acto II – Pas de deux', tier_name: 'Primer elenco', cast_label: 'A', default_sound_cue: null, notes: null,
    members: [] }] },
  { id: 'w2', title: 'El Cascanueces', composer: 'P. I. Tchaikovsky', choreographer: null, is_active: true, passages: [], casting_plans: [] }];
const docs = [1,2,3].map(k => ({ id: 'd' + k, title: 'Certificado médico ' + k, category: 'MEDICAL_CERTIFICATE', document_date: '2026-09-1' + k, personnel_id: 'p' + k, personnel_name: names[k], file_name: 'cert' + k + '.pdf', file_type: 'application/pdf', file_size_bytes: 120000, file_data: '', amount_ars: null, supplier_vendor: null, notes: 'Reposo 3 días', created_at: '', updated_at: '' }));
const soloists = [{ id: 'sc1', program_title: 'Temporada Giselle 2026', hall_name: 'Teatro del Libertador', document_date: '2026-09-30', recipient_name: 'Agencia Córdoba Cultura', recipient_title: 'Dirección', intro_body: 'Se certifica que los siguientes integrantes interpretaron roles solísticos.', closing_text: 'Sin otro particular.', status: 'DRAFT', notes: null, total_performances: 6, items: [{ id: 'i1', certification_id: 'sc1', personnel_id: 'p1', dancer_name: names[1], work_role: 'Giselle', performance_dates: '12, 13 y 14/9', performance_count: 3, sort_order: 0 }], created_at: '', updated_at: '' }];
const sumAgents = names.map((n, k) => ({ personnel_id: 'p' + k, legajo: String(1000 + k), dancer_name: n, role: 'DANCER', days_present: 18 - k % 4, days_late: k % 3, total_late_minutes: (k % 3) * 12, days_medical_leave: k % 5 === 0 ? 2 : 0, days_physio: k % 4 === 0 ? 3 : 0, days_absent: k % 6 === 0 ? 1 : 0, presence_percentage: 90 - k }));
export async function invoke(cmd: string, args: any = {}): Promise<any> {
  switch (cmd) {
    case 'get_or_create_daily_schedule': return { schedule: { ...schedule, schedule_date: args.scheduleDate }, conflicts: [] };
    case 'get_license_info': return { is_locked: false, warning: null, status: 'ACTIVE' };
    case 'get_monthly_bop_matrix': return matrix(args.year, args.month);
    case 'list_attendance_for_date': return att;
    case 'list_personnel': case 'search_dancer_quick': return pers;
    case 'list_stock_catalog': return stock;
    case 'get_agent_monthly_attendance_sheet': {
      const dim = 31; const dn = ['Dom','Lun','Mar','Mié','Jue','Vie','Sáb'];
      const entries = Array.from({ length: dim }, (_, i) => { const d = i + 1, wd = new Date(2026, 9, d).getDay();
        return { date: `2026-10-${String(d).padStart(2,'0')}`, day_number: d, day_of_week: dn[wd], is_weekend: wd === 0 || wd === 6, expected_checkin_time: '08:45', actual_checkin_time: d % 7 === 3 ? '09:02' : '08:40', minutes_late: d % 7 === 3 ? 17 : 0, status: d % 11 === 0 ? 'MEDICAL_LEAVE' : d % 7 === 3 ? 'LATE' : 'PRESENT', is_in_class: true, remarks: d % 9 === 0 ? 'Retiro anticipado por turno médico' : null, medical_diagnosis: d % 11 === 0 ? 'Lumbalgia' : null, has_certificate: d % 11 === 0 }; });
      return { personnel_id: 'p0', legajo: '1000', first_name: 'Ignacio', last_name: 'Altamirano', role: 'DANCER', category_name: 'Cuerpo de baile', year: 2026, month: 10, month_name: 'Octubre', total_working_days: 22, days_present: 18, days_late: 3, total_late_minutes: 51, days_medical_leave: 2, days_physiotherapy: 1, days_not_in_class: 0, days_absent: 0, presence_percentage: 91, entries };
    }
    case 'list_works_with_casts': return works;
    case 'list_soloist_certifications': return soloists;
    case 'get_soloist_certification': return soloists[0];
    case 'list_vault_documents': return docs;
    case 'get_vault_security_status': return { has_password: false, lock_timeout_minutes: 5 };
    case 'get_vault_monthly_totals': return { month: 9, year: 2026, total_amount_ars: 450000, receipts_count: 4, certificates_count: 3, documents_total_count: 7 };
    case 'get_lan_sync_info': return { local_ip: '192.168.100.20', port: 45870, is_server_active: true, device_name: 'Galaxy A53' };
    case 'get_lan_sync_key': return 'ABCD-1234';
    case 'list_monthly_agents_summary': return sumAgents;
    case 'list_daily_schedules': return [{ id: 'S1', schedule_date: schedule.schedule_date, regime: 'ORDINARY', mandatory_checkin_time: '08:45', nominal_end_time: '13:45', rehearsal_blocks_count: 1 }];
    default: console.log('mock invoke', cmd, args); return [];
  }
}
