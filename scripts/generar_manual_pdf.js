import PDFDocument from 'pdfkit';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const outputPath = process.argv[2] || path.join(__dirname, '..', 'Manual de Usuario TeatroGestion.pdf');

console.log(`Generando manual de usuario en: ${outputPath}`);

const doc = new PDFDocument({
    size: 'A4',
    margins: { top: 45, bottom: 45, left: 45, right: 45 },
    bufferPages: true,
    info: {
        Title: 'Manual de Usuario Oficial — TeatroGestión',
        Author: 'Ballet Oficial de Córdoba — Teatro del Libertador San Martín',
        Subject: 'Sistema Integral de Personal, Inventario, Repartos, Cues y Stock',
        Keywords: 'TeatroGestión, Ballet Oficial de Córdoba, Teatro del Libertador, Manual',
    }
});

const stream = fs.createWriteStream(outputPath);
doc.pipe(stream);

// Paleta Teatral Dark & Amber
const COLOR_BG_DARK = '#121214';
const COLOR_AMBER = '#D9822B'; // Ámbar legible para impresión
const COLOR_PRIMARY = '#1E293B';
const COLOR_SECONDARY = '#475569';
const COLOR_ACCENT_PURPLE = '#6B21A8';
const COLOR_BORDER = '#CBD5E1';
const COLOR_BOX_BG = '#F8FAFC';
const COLOR_TEXT = '#0F172A';

function addHeader() {
    doc.save();
    const oldBottom = doc.page.margins.bottom;
    doc.page.margins.bottom = 0;
    doc.fontSize(8).fillColor('#64748B').text('TEATROGESTIÓN — BALLET OFICIAL DE CÓRDOBA | TEATRO DEL LIBERTADOR SAN MARTÍN', 45, 25, { lineBreak: false });
    doc.fontSize(8).fillColor(COLOR_AMBER).text('★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★', 45, 25, { align: 'right', lineBreak: false });
    doc.moveTo(45, 38).lineTo(550, 38).strokeColor(COLOR_BORDER).lineWidth(0.5).stroke();
    doc.page.margins.bottom = oldBottom;
    doc.restore();
}

function addFooter(pageNum, totalPages) {
    doc.save();
    const oldBottom = doc.page.margins.bottom;
    doc.page.margins.bottom = 0;
    doc.moveTo(45, 805).lineTo(550, 805).strokeColor(COLOR_BORDER).lineWidth(0.5).stroke();
    doc.fontSize(8).fillColor('#64748B').text('Manual de Usuario y Operación Técnica — Windows 11 & Android', 45, 812, { lineBreak: false });
    doc.fontSize(8).fillColor('#64748B').text(`Página ${pageNum} de ${totalPages}`, 45, 812, { align: 'right', lineBreak: false });
    doc.page.margins.bottom = oldBottom;
    doc.restore();
}

function sectionTitle(title, number) {
    doc.moveDown(1.2);
    doc.rect(45, doc.y, 4, 18).fill(COLOR_AMBER);
    doc.fontSize(14).font('Helvetica-Bold').fillColor(COLOR_PRIMARY).text(`  ${number}. ${title}`, 55, doc.y + 2);
    doc.moveDown(0.6);
}

function subTitle(title) {
    doc.moveDown(0.8);
    doc.fontSize(11).font('Helvetica-Bold').fillColor(COLOR_SECONDARY).text(title);
    doc.moveDown(0.4);
}

function paragraph(text) {
    doc.fontSize(9.5).font('Helvetica').fillColor(COLOR_TEXT).text(text, { align: 'justify', lineGap: 2.5 });
    doc.moveDown(0.5);
}

function bullet(title, text) {
    doc.fontSize(9.5).font('Helvetica-Bold').fillColor(COLOR_PRIMARY).text(`• ${title}: `, { continued: true });
    doc.font('Helvetica').fillColor(COLOR_TEXT).text(text, { align: 'justify', lineGap: 2 });
    doc.moveDown(0.3);
}

function callout(title, text, borderColor = COLOR_AMBER, bgColor = COLOR_BOX_BG) {
    doc.moveDown(0.4);
    const startY = doc.y;
    const boxWidth = 505;
    
    // Altura tentativa
    doc.fontSize(9).font('Helvetica');
    const textHeight = doc.heightOfString(text, { width: boxWidth - 24, lineGap: 2 });
    const totalHeight = textHeight + 28;

    doc.rect(45, startY, boxWidth, totalHeight).fill(bgColor);
    doc.rect(45, startY, 4, totalHeight).fill(borderColor);
    doc.rect(45, startY, boxWidth, totalHeight).strokeColor(COLOR_BORDER).lineWidth(0.5).stroke();

    doc.fontSize(9.5).font('Helvetica-Bold').fillColor(borderColor).text(title, 57, startY + 6);
    doc.fontSize(9).font('Helvetica').fillColor(COLOR_TEXT).text(text, 57, startY + 20, { width: boxWidth - 24, align: 'justify', lineGap: 2 });
    
    doc.y = startY + totalHeight + 10;
}

// ==========================================
// PORTADA
// ==========================================
doc.rect(0, 0, 595.28, 841.89).fill('#0F172A');

doc.rect(45, 120, 505, 10).fill('#FFB31A');

doc.fontSize(32).font('Helvetica-Bold').fillColor('#FFFFFF').text('TEATROGESTIÓN', 45, 150, { characterSpacing: 1.5 });
doc.fontSize(14).font('Helvetica').fillColor('#94A3B8').text('SISTEMA INTEGRAL DE GESTIÓN TÉCNICA, PERSONAL Y REPARTOS', 45, 190);
doc.fontSize(12).font('Helvetica-Bold').fillColor('#FFB31A').text('BALLET OFICIAL DE CÓRDOBA — TEATRO DEL LIBERTADOR SAN MARTÍN', 45, 210);

doc.rect(45, 240, 505, 1).fill('#334155');

doc.fontSize(16).font('Helvetica-Bold').fillColor('#F8FAFC').text('MANUAL OFICIAL DE USUARIO Y OPERACIÓN', 45, 270);
doc.fontSize(11).font('Helvetica').fillColor('#CBD5E1').text('Guía Integral para Inspección de Ballet, Dirección, Regiduría y Jefatura de Sala', 45, 295);

doc.rect(45, 340, 505, 120).fill('#1E293B');
doc.rect(45, 340, 5, 120).fill('#FFB31A');

doc.fontSize(11).font('Helvetica-Bold').fillColor('#FFB31A').text('CLÁUSULA INSTITUCIONAL PERMANENTE:', 60, 355);
doc.fontSize(13).font('Helvetica-Bold').fillColor('#FFFFFF').text('«★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD', 60, 378);
doc.fontSize(13).font('Helvetica-Bold').fillColor('#FFFFFF').text('  Y SUJETO A CAMBIOS. ★»', 60, 396);
doc.fontSize(9.5).font('Helvetica-Oblique').fillColor('#94A3B8').text('Texto legal y operativo que rige toda la documentación oficial del Ballet Oficial de Córdoba.', 60, 422);

doc.fontSize(10).font('Helvetica').fillColor('#94A3B8').text('Áreas cubiertas en este manual:', 45, 490);
const topics = [
    'Instalador Dual en Windows 11 (Interfaz Gráfica WPF & Modo Consola CLI)',
    'Recuperación Administrativa de Contraseña con Llave Maestra (master.key.enc)',
    'Padrón del Personal con Documento Nacional de Identidad (DNI)',
    'Régimen Diario de Ensayos y Descansos Obligatorios (15 min)',
    'Francos Compensatorios Configurables (1 a 4 días tras funciones)',
    'Control de Ausentismo en Clase, Fisioterapia y Estado "Sin Parte" (S/P)',
    'Matriz Mensual de Asistencia del BOP con Códigos Oficiales',
    'Planificación de Repartos y Certificaciones de Roles de Solistas',
    'Bóveda Documental de Inspección: Certificados Médicos y Gastos de Caja Chica ($ ARS)',
    'Sincronización P2P en Red Local (LAN) y Aplicación Móvil para Android'
];

let ty = 510;
for (const t of topics) {
    doc.fontSize(9).font('Helvetica-Bold').fillColor('#FFB31A').text('• ', 45, ty, { continued: true });
    doc.font('Helvetica').fillColor('#E2E8F0').text(t);
    ty += 16;
}

doc.fontSize(9).font('Helvetica').fillColor('#64748B').text('Versión 1.0 (Producción 2026) | Entorno Windows 11 / Android | Teatro del Libertador San Martín', 45, 760);

// ==========================================
// PÁGINA 2: INTRODUCCIÓN Y ARQUITECTURA
// ==========================================
doc.addPage();
addHeader();

sectionTitle('Introducción y Arquitectura Offline', 1);

paragraph('TeatroGestión es la plataforma integral de software de misión crítica diseñada específicamente para responder a las exigencias operativas y reglamentarias del Ballet Oficial de Córdoba en el Teatro del Libertador General San Martín.');

paragraph('El sistema ha sido concebido bajo el principio de "Soberanía Offline Absoluta". En los teatros históricos y salas de ensayo provinciales, la conectividad a internet es a menudo inestable, inexistente o restringida por políticas de red institucional. Por ello, TeatroGestión opera 100% de manera autónoma, almacenando todos sus registros en una base de datos local SQLite con journaling WAL (Write-Ahead Logging) de alto rendimiento.');

subTitle('Pilares de Seguridad y Criptografía');
bullet('Identificación Física (Machine ID)', 'Cada instalación vincula su funcionamiento al hardware de la PC del teatro mediante un identificador criptográfico único extraído de la placa madre y el sistema operativo.');
bullet('Licencias Offline Ed25519', 'Las licencias se emiten y firman mediante criptografía asimétrica de curva elíptica Ed25519 utilizando la llave maestra del desarrollador ("master.key.enc").');
bullet('Reloj Monotónico Antifraude', 'El sistema detecta manipulaciones intencionales del reloj de Windows impidiendo el adelantamiento o atraso ficticio de fechas mediante marcas en disco.');
bullet('Base de Datos Robusta SQLite WAL', 'Garantiza consistencia ACID completa, tolerancia a cortes intempestivos de energía eléctrica y cero corrupción de archivos.');

callout('REGLA DE ORO OPERATIVA: CERO CONECTIVIDAD OBLIGATORIA', 
'TeatroGestión jamás depende de servidores en la nube ni telemetría externa. La inspectora, el director y los técnicos pueden gestionar personal, inventario, ausentismo y repartos en la cabina o sala sin requerir internet.');

// ==========================================
// PÁGINA 3: INSTALACIÓN DUAL WINDOWS 11
// ==========================================
doc.addPage();
addHeader();

sectionTitle('Instalación en Windows 11 (Opciones Duales)', 2);

paragraph('El pendrive de instalación de 64 GB incluye dos alternativas de ejecución diseñadas para adaptarse a la preferencia del operador:');

subTitle('Opción A: Instalador Gráfico Moderno (Recomendado)');
paragraph('Al ejecutar "Instalar_TeatroGestion_Grafico.bat" (o invocar "Instalar_TeatroGestion.exe --gui"), se abre la interfaz gráfica moderna de Windows 11 con la estética TeatroCue Dark Graphite:');
bullet('Selector de Carpeta', 'Propone por defecto "C:\\TeatroGestion" o la carpeta de usuario local, con botón "Examinar..."');
bullet('Selector de Licencia', 'Permite elegir con un clic: 1 Año (Recomendado), 1 Mes, 3 Meses, 6 Meses, Perpetua o Días personalizados.');
bullet('Puesto / Sala', 'Permite identificar la estación de trabajo (ej: "Cabina / Oficina", "Inspección de Ballet").');
bullet('Contraseña Maestra', 'Campo protegido tipo PasswordBox para descifrar en memoria RAM la llave "master.key.enc".');
bullet('Configuración de Bóveda', 'Permite definir la contraseña inicial de la inspectora y el tiempo de bloqueo por inactividad.');
bullet('Consola de Registro en Vivo', 'Muestra en tiempo real cada paso de la instalación con barra de progreso animada.');

subTitle('Opción B: Asistente Interactivo de Consola (CLI)');
paragraph('Al ejecutar "Instalar_TeatroGestion_Consola.bat" (o invocar "Instalar_TeatroGestion.exe --cli"), se presenta un asistente interactivo en terminal con menú numerado:');
bullet('Opción [1]', 'Instalar y Activar TeatroGestión paso a paso.');
bullet('Opción [2]', 'Lanzar el Instalador Gráfico.');
bullet('Opción [3]', 'Restablecer Contraseña de la Bóveda de Inspección (Modo Administrador).');
bullet('Opción [4]', 'Salir del instalador.');

callout('INSTALACIÓN RÁPIDA DESDE EL PENDRIVE', 
'Para instalar en una PC nueva del teatro, simplemente inserte el pendrive USB, haga doble clic en "Instalar_TeatroGestion_Grafico.bat", ingrese la contraseña maestra y presione [ INSTALAR Y ACTIVAR TEATROGESTIÓN ]. En 5 segundos el programa estará listo en el Escritorio.');

// ==========================================
// PÁGINA 4: RECUPERACIÓN DE CONTRASEÑA ADMIN
// ==========================================
doc.addPage();
addHeader();

sectionTitle('Recuperación Administrativa de la Bóveda', 3);

paragraph('Si la inspectora (Carina Lion) o el personal autorizado olvida la contraseña de la Bóveda Documental, el Administrador del sistema que cuenta con el pendrive de instalación puede restablecerla en el acto mediante la llave maestra.');

subTitle('Procedimiento desde el Instalador Gráfico:');
bullet('Paso 1', 'Conectar el pendrive USB a la computadora.');
bullet('Paso 2', 'Abrir "Instalar_TeatroGestion_Grafico.bat" y seleccionar la pestaña "Recuperar Contraseña de Bóveda".');
bullet('Paso 3', 'Verificar que la ruta de la base de datos detectada sea "%APPDATA%\\com.teatrogestion.app\\teatro_gestion.sqlite".');
bullet('Paso 4', 'Ingresar la Contraseña Maestra del Administrador (la misma utilizada para firmar licencias).');
bullet('Paso 5', 'Ingresar y confirmar la NUEVA contraseña para la inspectora, junto con el tiempo de bloqueo deseado.');
bullet('Paso 6', 'Hacer clic en [ RESTABLECER CONTRASEÑA DE BÓVEDA ].');

subTitle('Procedimiento desde la Consola Interactiva:');
bullet('Ejecución', 'Ejecutar "Instalar_TeatroGestion_Consola.bat", elegir la opción [3] (o ejecutar "Instalar_TeatroGestion.exe --admin-reset-vault").');
bullet('Validación', 'El sistema solicita la contraseña maestra del desarrollador/administrador. Si es correcta, solicita la nueva clave para la inspectora y actualiza inmediatamente la base de datos SQLite.');

callout('BLINDAJE DE SEGURIDAD', 
'Nadie ajeno puede alterar ni resetear la clave de la inspectora sin poseer físicamente el pendrive con el archivo "master.key.enc" y conocer la contraseña maestra del desarrollador.');

// ==========================================
// PÁGINA 5: PADRÓN DE PERSONAL Y RÉGIMEN
// ==========================================
doc.addPage();
addHeader();

sectionTitle('Padrón del Personal y Documento de Identidad', 4);

paragraph('El Padrón Oficial centraliza la totalidad de los recursos humanos del cuerpo artístico y técnico. Cada integrante cuenta con su ficha individual que incluye:');
bullet('Identificación', 'Nombre y apellido oficial, alias artístico y Documento Nacional de Identidad (DNI) obligatorio para partes legales.');
bullet('Rol Institucional', 'Bailarín/a, Maestro/a de Baile, Kinesiólogo/a, Inspector/a, Director/a o Asistente Técnico.');
bullet('Estado en el Elenco', 'Activo, Licencia Médica, Gira o Pasivo.');

sectionTitle('Régimen Horario y Francos Compensatorios', 5);

paragraph('El Ballet Oficial de Córdoba se rige por normas estrictas de descanso y regímenes de trabajo pactados internamente con Dirección y Supervisión:');
subTitle('1. Intervalo de Descanso Reglamentario (15 minutos)');
paragraph('Entre la finalización de la clase matutina y el inicio del primer bloque de ensayo general, el sistema valida y exige un intervalo de descanso mínimo de 15 minutos para permitir la hidratación y recuperación muscular de los bailarines.');

subTitle('2. Francos Compensatorios Configurables Post-Función');
paragraph('Tras funciones oficiales de temporada o funciones de gira en el interior provincial, la inspectora y Dirección acuerdan internamente los días de franco compensatorio:');
bullet('Selector de Compensatorios', 'Directamente desde la barra superior de la app ("Régimen Diario"), la inspectora puede seleccionar entre: 1 día, 2 días, 3 días o 4 días de franco compensatorio.');
bullet('Visualización Táctica', 'El sistema muestra un distintivo en la barra superior indicando los días de compensatorio vigentes.');

// ==========================================
// PÁGINA 6: AUSENTISMO Y SIN PARTE (S/P)
// ==========================================
doc.addPage();
addHeader();

sectionTitle('Control de Ausentismo y Estado "Sin Parte" (S/P)', 6);

paragraph('El control de ausentismo del Ballet Oficial de Córdoba no se realiza por fichado biométrico de entrada sino durante la clase matutina en la sala de ensayo. La inspectora recorre la barra registrando las presencias y motivos de inasistencia.');

subTitle('El Nuevo Estado Oficial: "Sin Parte" (S/P)');
paragraph('En los días de función o ensayos parciales, ciertos bailarines del padrón no son convocados para el repertorio de la jornada. Anteriormente, esto generaba ambigüedad en los registros.');
bullet('Código Táctico S/P', 'Indica que el agente se encuentra "Sin parte" oficial de ensayo (no convocado por la Dirección).');
bullet('Cómputo Justo', 'En la Matriz Mensual de Asistencia del BOP, el estado "S/P" no computa como falta injustificada ni afecta el presentismo del bailarín.');
bullet('Color Distintivo', 'En el panel táctico de ausentismo y en la matriz, se identifica con color púrpura (#B388FF).');

subTitle('Matriz de Asistencia Mensual del BOP');
paragraph('Permite visualizar la grilla completa del mes con todos los integrantes del ballet y los 31 días. Cada celda permite ciclar los estados oficiales con un simple clic:');
bullet('P', 'Presente en clase y ensayo.');
bullet('T', 'Tarde justificada.');
bullet('J', 'Inasistencia Justificada (aviso previo).');
bullet('M', 'Licencia Médica oficial.');
bullet('G', 'Licencia por Gira artística.');
bullet('I', 'Inasistencia Injustificada.');
bullet('S/P', 'Sin parte (no convocado).');

// ==========================================
// PÁGINA 7: BÓVEDA DE INSPECCIÓN (CARINA LION)
// ==========================================
doc.addPage();
addHeader();

sectionTitle('Bóveda Documental de Inspección', 7);

paragraph('La Bóveda Documental de Inspección es el espacio privado y seguro exclusivo de la inspectora (Carina Lion) para resguardar documentación respaldatoria sensible que no debe estar expuesta al público.');

subTitle('Características de Seguridad');
bullet('PIN / Contraseña Salteada', 'Protegida mediante hash criptográfico SHA-256 con salteado interno. La clave se define en la instalación y puede cambiarse dentro de la app.');
bullet('Temporizador de Inactividad', 'Se bloquea automáticamente tras 1, 5, 15, 30 minutos de inactividad, o puede dejarse en "Nunca bloquear" si la inspectora así lo prefiere.');
bullet('Desbloqueo Rápido', 'Permite introducir la contraseña con la tecla Enter para desbloqueo instantáneo.');

subTitle('Gestión Documental y Caja Chica');
bullet('Certificados Médicos', 'Fotografías o escaneos de certificados de reposo, diagnósticos traumatológicos y partes de kinesiología.');
bullet('Caja Chica y Compras Urgentes', 'Registro de compras menores indispensables (colofonia en panes, cintas kinesiológicas, vendas elásticas, agujas para zapatillas). Permite registrar Monto en $ ARS y Comercio/Proveedor.');
bullet('Totalizador Automático', 'La bóveda suma y totaliza en tiempo real los gastos del mes en pesos argentinos, facilitando la rendición de cuentas ante Administración.');
bullet('Memos Oficiales de Dirección', 'Órdenes de servicio y acuerdos de ensayo emitidos a horas intempestivas (ej: 22:52 hs).');

// ==========================================
// PÁGINA 8: PLANIFICACIÓN Y REPARTOS
// ==========================================
doc.addPage();
addHeader();

sectionTitle('Planificación de Repartos y Solísticos', 8);

paragraph('El módulo de repartos permite armar el esquema coreográfico de la temporada garantizando el cumplimiento de la cláusula de reparto de estudio:');
bullet('Obras y Pasajes', 'Configuración de obras del repertorio (ej: La Sylphide, El Lago de los Cisnes, Paquita, Giselle) y sus pasajes (Pas de Deux, Solos, Cuerpo de Baile).');
bullet('Asignación desde el Padrón', 'Al editar un solístico o reparto, el sistema despliega el padrón oficial para vincular directamente los datos del bailarín sin necesidad de reescribirlos.');
bullet('Certificación de Solísticos', 'Generación de certificados de roles principales y de solista, útiles para concursos de jerarquización y antecedentes curriculares.');

sectionTitle('Sincronización LAN y Versión Móvil Android', 9);

paragraph('Para operar en giras o salas desconectadas sin depender de la nube:');
bullet('Sincronización P2P en Red Local', 'Las computadoras conectadas a la misma red WiFi o cable Ethernet del teatro pueden sincronizar datos entre sí en un clic sin requerir internet.');
bullet('Paquetes Portables (.tgpack)', 'Permite exportar un archivo de respaldo cifrado y comprimido con Zstandard para trasladarlo mediante el pendrive a otra máquina.');
bullet('App Móvil para Android (APK)', 'El pendrive incluye "TeatroGestion.apk" para instalar en teléfonos Samsung Galaxy A53 o tablets de inspección. Permite escaneo con cámara del código de barras de las zapatillas de punta Gaynor Minden.');

callout('CONTACTO Y SOPORTE TÉCNICO', 
'Para reseteo de emergencias, extensión de licencias o asistencia operativa, comuníquese con el desarrollador del sistema del Ballet Oficial de Córdoba en el Teatro del Libertador San Martín.');

// Finalizar Páginas y Numeración
const totalPages = doc.bufferedPageRange().count;
for (let i = 1; i < totalPages; i++) {
    doc.switchToPage(i);
    addFooter(i + 1, totalPages);
}

doc.end();

stream.on('finish', () => {
    console.log(`[OK] Manual en PDF generado exitosamente: ${outputPath} (${totalPages} páginas)`);
});
