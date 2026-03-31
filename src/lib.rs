use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcesamientoError {
    #[error("Error CSV: {0}")]
    Csv(#[from] csv::Error),
    
    #[error("Error fecha: {0}")]
    Fecha(String),
    
    #[error("Error IO: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistroProcesado {
    pub entry_id: u32,
    pub co2_ppm: f64,
    pub ch4_ppm: f64,
    pub created_at: DateTime<FixedOffset>,
    pub fecha: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct RegistroDiario {
    pub entry_id: u32,
    pub co2_ppm: f64,
    pub ch4_ppm: f64,
    pub created_at: DateTime<FixedOffset>,
    pub tiempo_relativo_min: f64,
}

#[derive(Debug, Clone)]
pub struct PendienteDia {
    pub fecha: NaiveDate,
    pub pendiente_co2: f64,
    pub pendiente_ch4: f64,
    pub r2_co2: f64,
    pub r2_ch4: f64,
}

pub fn parsear_fecha(fecha_str: &str) -> Result<DateTime<FixedOffset>, ProcesamientoError> {
    DateTime::parse_from_rfc3339(fecha_str)
        .map_err(|e| ProcesamientoError::Fecha(format!("Error parseando '{}': {}", fecha_str, e)))
}

pub fn extraer_fecha(date_time: DateTime<FixedOffset>) -> NaiveDate {
    date_time.date_naive()
}

pub fn calcular_tiempo_relativo(
    registros: &[RegistroProcesado],
) -> Vec<RegistroDiario> {
    if registros.is_empty() {
        return vec![];
    }

    let primer_registro = registros[0].created_at;
    
    registros
        .iter()
        .map(|r| {
            let duracion = r.created_at.signed_duration_since(primer_registro);
            let tiempo_min = duracion.num_seconds() as f64 / 60.0;
            
            RegistroDiario {
                entry_id: r.entry_id,
                co2_ppm: r.co2_ppm,
                ch4_ppm: r.ch4_ppm,
                created_at: r.created_at,
                tiempo_relativo_min: tiempo_min,
            }
        })
        .collect()
}

pub fn calcular_pendiente_regresion_lineal(
    x: &[f64],
    y: &[f64],
) -> (f64, f64) {
    let n = x.len() as f64;
    if n == 0.0 {
        return (0.0, 0.0);
    }

    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    let sum_x2: f64 = x.iter().map(|xi| xi * xi).sum();

    let pendiente = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
    
    let num = n * sum_xy - sum_x * sum_y;
    let den = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2(&y) - sum_y * sum_y)).sqrt();
    
    let r = if den == 0.0 { 0.0 } else { num / den };
    let r2 = r * r;

    (pendiente, r2)
}

fn sum_y2(y: &[f64]) -> f64 {
    y.iter().map(|yi| yi * yi).sum()
}

pub fn es_valor_saturacion(valor: f64) -> bool {
    (valor - 6000.0).abs() < 0.5
}

pub fn filtrar_saturacion(registros: Vec<RegistroProcesado>) -> Vec<RegistroProcesado> {
    let mut filtrados = Vec::new();
    let mut en_saturacion = false;
    
    for registro in registros {
        let co2_saturado = es_valor_saturacion(registro.co2_ppm);
        let ch4_saturado = es_valor_saturacion(registro.ch4_ppm);
        
        if co2_saturado || ch4_saturado {
            en_saturacion = true;
            continue;
        }
        
        if en_saturacion {
            if registro.co2_ppm < 1000.0 && registro.ch4_ppm < 1000.0 {
                en_saturacion = false;
                filtrados.push(registro);
            }
        } else {
            filtrados.push(registro);
        }
    }
    
    filtrados
}