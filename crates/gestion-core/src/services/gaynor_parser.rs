use crate::domain::{
    GaynorBox, GaynorFit, GaynorHeel, GaynorParsedSpec, GaynorShank, GaynorVamp, GaynorWidth,
};
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum GaynorParseError {
    #[error("El código Gaynor Minden está vacío")]
    EmptyCode,
    #[error("Prefijo de modelo (Fit) inválido: '{0}'. Valores admitidos: SC (Sculpted), CL (Classic), SK (Sleek)")]
    InvalidFit(String),
    #[error("Talle numérico inválido: '{0}'. Debe ser un número entre 3.0 y 15.0 en pasos de 0.5")]
    InvalidSize(String),
    #[error("Ancho inválido: '{0}'. Valores admitidos: N (Narrow), M (Medium), W (Wide)")]
    InvalidWidth(char),
    #[error("Número de caja inválido: '{0}'. Valores admitidos: 2, 3, 4, 5")]
    InvalidBox(char),
    #[error("Dureza de vástago/bolsa inválida: '{0}'. Valores admitidos: X (Extraflex), S (Supple), F (Feather), P (Pianissimo), H (Hard)")]
    InvalidShank(char),
    #[error("Código de capellada (Vamp) inválido: '{0}'. Valores admitidos: D (Deep), L (Low)")]
    InvalidVamp(char),
    #[error("Código de talón (Heel) inválido: '{0}'. Valores admitidos: H (High), L (Low)")]
    InvalidHeel(char),
    #[error("Estructura de código Gaynor Minden incompleta o malformada: '{0}'")]
    MalformedSpecification(String),
}

/// Parsea y descompone códigos alfanuméricos de fábrica de puntas Gaynor Minden:
/// - `"SC - 7.5W4PDH"` -> Sculpted, Talle 7.5, Ancho W, Caja 4, Shank Pianissimo (P), Vamp Deep (D), Heel High (H)
/// - `"SK - 8.5M4HDL"` -> Sleek, Talle 8.5, Ancho M, Caja 4, Shank Hard (H), Vamp Deep (D), Heel Low (L)
/// - `"CL - 8M3XDH"`   -> Classic, Talle 8, Ancho M, Caja 3, Shank Extraflex (X), Vamp Deep (D), Heel High (H)
/// - Soporta variantes de capellada alta adicional: `"+6mm"` (`6 mm`) o `"+1/2\""` (`1/2"`).
pub fn parse_gaynor_code(raw_input: &str) -> Result<GaynorParsedSpec, GaynorParseError> {
    let trimmed = raw_input.trim();
    if trimmed.is_empty() {
        return Err(GaynorParseError::EmptyCode);
    }

    let upper = trimmed.to_uppercase();

    // 1. Detectar sufijos de capellada alta especial (+6mm o +1/2")
    let (base_part, vamp_override) = if upper.contains("6MM") || upper.contains("6 MM") {
        let cleaned = upper
            .replace("+6MM", "")
            .replace("+ 6MM", "")
            .replace("+6 MM", "")
            .replace("6MM", "")
            .replace("6 MM", "")
            .replace(['(', ')'], "");
        (cleaned, Some(GaynorVamp::DeepPlus6Mm))
    } else if upper.contains("1/2\"") || upper.contains("1/2") {
        let cleaned = upper
            .replace("+1/2\"", "")
            .replace("+ 1/2\"", "")
            .replace("1/2\"", "")
            .replace("+1/2", "")
            .replace("1/2", "")
            .replace(['(', ')'], "");
        (cleaned, Some(GaynorVamp::DeepPlusHalfInch))
    } else {
        (upper, None)
    };

    // 2. Eliminar espacios en blanco y dividir entre prefijo Fit (SC, CL, SK) y cuerpo técnico
    let compact: String = base_part.chars().filter(|c| !c.is_whitespace()).collect();
    let (fit_str, spec_body) = if let Some((left, right)) = compact.split_once('-') {
        (left, right)
    } else if compact.len() >= 8 {
        (&compact[..2], &compact[2..])
    } else {
        return Err(GaynorParseError::MalformedSpecification(raw_input.to_string()));
    };

    let fit = match fit_str {
        "SC" => GaynorFit::Sculpted,
        "CL" => GaynorFit::Classic,
        "SK" => GaynorFit::Sleek,
        other => return Err(GaynorParseError::InvalidFit(other.to_string())),
    };

    // 3. Extraer el talle numérico al inicio del cuerpo (ej. "7.5", "8.5", "8", "10.5")
    let size_end_idx = spec_body
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .ok_or_else(|| GaynorParseError::MalformedSpecification(raw_input.to_string()))?;

    if size_end_idx == 0 {
        return Err(GaynorParseError::MalformedSpecification(raw_input.to_string()));
    }

    let size_str = &spec_body[..size_end_idx];
    let size_num: f32 = size_str
        .parse()
        .map_err(|_| GaynorParseError::InvalidSize(size_str.to_string()))?;

    let doubled = (size_num * 2.0).round();
    let is_half_step = ((size_num * 2.0) - doubled).abs() < 1e-4;
    if !(3.0..=15.0).contains(&size_num) || !is_half_step {
        return Err(GaynorParseError::InvalidSize(size_str.to_string()));
    }

    // 4. Descomponer los 5 caracteres restantes: [Width][Box][Shank][Vamp][Heel]
    let tail: Vec<char> = spec_body[size_end_idx..].chars().collect();
    if tail.len() != 5 {
        return Err(GaynorParseError::MalformedSpecification(raw_input.to_string()));
    }

    let width = match tail[0] {
        'N' => GaynorWidth::Narrow,
        'M' => GaynorWidth::Medium,
        'W' => GaynorWidth::Wide,
        c => return Err(GaynorParseError::InvalidWidth(c)),
    };

    let box_shape = match tail[1] {
        '2' => GaynorBox::Box2,
        '3' => GaynorBox::Box3,
        '4' => GaynorBox::Box4,
        '5' => GaynorBox::Box5,
        c => return Err(GaynorParseError::InvalidBox(c)),
    };

    let shank = match tail[2] {
        'X' => GaynorShank::Extraflex,
        'S' => GaynorShank::Supple,
        'F' => GaynorShank::Feather,
        'P' => GaynorShank::Pianissimo,
        'H' => GaynorShank::Hard,
        c => return Err(GaynorParseError::InvalidShank(c)),
    };

    let base_vamp = match tail[3] {
        'D' => GaynorVamp::Deep,
        'L' => GaynorVamp::Low,
        c => return Err(GaynorParseError::InvalidVamp(c)),
    };

    let vamp = match vamp_override {
        Some(special_vamp) => {
            if base_vamp != GaynorVamp::Deep {
                return Err(GaynorParseError::InvalidVamp(tail[3]));
            }
            special_vamp
        }
        None => base_vamp,
    };

    let heel = match tail[4] {
        'H' => GaynorHeel::High,
        'L' => GaynorHeel::Low,
        c => return Err(GaynorParseError::InvalidHeel(c)),
    };

    Ok(GaynorParsedSpec {
        raw_code: trimmed.to_string(),
        fit,
        size: size_str.to_string(),
        width,
        box_shape,
        shank,
        vamp,
        heel,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_parse_official_codes_from_spec() {
        let sc = parse_gaynor_code("SC - 7.5W4PDH").expect("Debe parsear SC - 7.5W4PDH");
        assert_eq!(sc.fit, GaynorFit::Sculpted);
        assert_eq!(sc.size, "7.5");
        assert_eq!(sc.width, GaynorWidth::Wide);
        assert_eq!(sc.box_shape, GaynorBox::Box4);
        assert_eq!(sc.shank, GaynorShank::Pianissimo);
        assert_eq!(sc.vamp, GaynorVamp::Deep);
        assert_eq!(sc.heel, GaynorHeel::High);
        assert_eq!(sc.canonical_code(), "SC - 7.5W4PDH");

        let sk = parse_gaynor_code("SK - 8.5M4HDL").expect("Debe parsear SK - 8.5M4HDL");
        assert_eq!(sk.fit, GaynorFit::Sleek);
        assert_eq!(sk.size, "8.5");
        assert_eq!(sk.width, GaynorWidth::Medium);
        assert_eq!(sk.box_shape, GaynorBox::Box4);
        assert_eq!(sk.shank, GaynorShank::Hard);
        assert_eq!(sk.vamp, GaynorVamp::Deep);
        assert_eq!(sk.heel, GaynorHeel::Low);
        assert_eq!(sk.canonical_code(), "SK - 8.5M4HDL");

        let cl = parse_gaynor_code("CL - 8M3XDH").expect("Debe parsear CL - 8M3XDH");
        assert_eq!(cl.fit, GaynorFit::Classic);
        assert_eq!(cl.size, "8");
        assert_eq!(cl.width, GaynorWidth::Medium);
        assert_eq!(cl.box_shape, GaynorBox::Box3);
        assert_eq!(cl.shank, GaynorShank::Extraflex);
        assert_eq!(cl.vamp, GaynorVamp::Deep);
        assert_eq!(cl.heel, GaynorHeel::High);
        assert_eq!(cl.canonical_code(), "CL - 8M3XDH");
    }

    #[test]
    fn test_parse_high_vamp_variants_6mm_and_half_inch() {
        let v_6mm = parse_gaynor_code("SC - 7.5W4PDH +6mm").unwrap();
        assert_eq!(v_6mm.vamp, GaynorVamp::DeepPlus6Mm);
        assert_eq!(v_6mm.canonical_code(), "SC - 7.5W4PDH +6mm");

        let v_6mm_spaced = parse_gaynor_code("sc-7.5w4pdh (6 mm)").unwrap();
        assert_eq!(v_6mm_spaced.vamp, GaynorVamp::DeepPlus6Mm);

        let v_half = parse_gaynor_code("SC - 8W4PDH +1/2\"").unwrap();
        assert_eq!(v_half.vamp, GaynorVamp::DeepPlusHalfInch);
        assert_eq!(v_half.canonical_code(), "SC - 8W4PDH +1/2\"");
    }

    #[test]
    fn test_reject_invalid_codes() {
        assert_eq!(parse_gaynor_code("   "), Err(GaynorParseError::EmptyCode));
        assert_eq!(
            parse_gaynor_code("ZZ - 7.5W4PDH"),
            Err(GaynorParseError::InvalidFit("ZZ".to_string()))
        );
        assert_eq!(
            parse_gaynor_code("SC - 7.5Q4PDH"),
            Err(GaynorParseError::InvalidWidth('Q'))
        );
        assert_eq!(
            parse_gaynor_code("SC - 7.5W9PDH"),
            Err(GaynorParseError::InvalidBox('9'))
        );
        assert_eq!(
            parse_gaynor_code("SC - 7.5W4ZDH"),
            Err(GaynorParseError::InvalidShank('Z'))
        );
    }

    proptest! {
        #[test]
        fn prop_gaynor_canonical_roundtrip(
            fit_idx in 0usize..3,
            half_steps in 10u32..24, // talles 5.0 a 11.5
            width_idx in 0usize..3,
            box_idx in 0usize..4,
            shank_idx in 0usize..5,
            vamp_idx in 0usize..4,
            heel_idx in 0usize..2,
        ) {
            let fits = ["SC", "CL", "SK"];
            let widths = ['N', 'M', 'W'];
            let boxes = ['2', '3', '4', '5'];
            let shanks = ['X', 'S', 'F', 'P', 'H'];
            let heels = ['H', 'L'];

            let size_str = if half_steps % 2 == 0 {
                format!("{}", half_steps / 2)
            } else {
                format!("{}.5", half_steps / 2)
            };

            let (vamp_char, suffix) = match vamp_idx {
                0 => ('L', ""),
                1 => ('D', ""),
                2 => ('D', " +6mm"),
                _ => ('D', " +1/2\""),
            };

            let code = format!(
                "{} - {}{}{}{}{}{}{}",
                fits[fit_idx],
                size_str,
                widths[width_idx],
                boxes[box_idx],
                shanks[shank_idx],
                vamp_char,
                heels[heel_idx],
                suffix
            );

            let parsed = parse_gaynor_code(&code).expect("Debe parsear código generado");
            prop_assert_eq!(parsed.canonical_code(), code);
        }
    }
}
