use proyecto_gases_hi::{RegistroProcesado, calcular_tiempo_relativo, RegistroDiario};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use csv::{ReaderBuilder, WriterBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 02_SEGMENTACION_DIARIA ===\n");
    
    let input_path = "data/processed/datos_limpios.csv";
    let output_dir = "data/processed/datos_por_dia";
    
    if !Path::new(output_dir).exists() {
        fs::create_dir_all(output_dir)?;
    }
    
    println!("📖 Leyendo: {}", input_path);
    
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(input_path)?;
    
    let mut por_dia: HashMap<String, Vec<RegistroProcesado>> = HashMap::new();
    
    for result in reader.records() {
        let record = result?;
        
        let entry_id: u32 = record.get(0).unwrap_or("0").parse().unwrap_or(0);
        let co2_ppm: f64 = record.get(1).unwrap_or("0").parse().unwrap_or(0.0);
        let ch4_ppm: f64 = record.get(2).unwrap_or("0").parse().unwrap_or(0.0);
        let created_at = proyecto_gases_hi::parsear_fecha(record.get(3).unwrap_or(""))?;
        let fecha = proyecto_gases_hi::extraer_fecha(created_at);
        
        let fecha_str = fecha.format("%Y-%m-%d").to_string();
        
        por_dia.entry(fecha_str).or_insert_with(Vec::new).push(RegistroProcesado {
            entry_id,
            co2_ppm,
            ch4_ppm,
            created_at,
            fecha,
        });
    }
    
    println!("✓ Días encontrados: {}", por_dia.len());
    
    let mut total_registros_guardados = 0;
    
    for (fecha, mut registros) in &mut por_dia {
        registros.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        
        let registros_diarios = calcular_tiempo_relativo(registros);
        
        let output_path = format!("{}/{}.csv", output_dir, fecha);
        let mut writer = WriterBuilder::new()
            .has_headers(true)
            .from_path(&output_path)?;
        
        writer.write_record(&["entry_id", "co2_ppm", "ch4_ppm", "created_at", "tiempo_min"])?;
        
        for reg in &registros_diarios {
            writer.write_record(&[
                reg.entry_id.to_string(),
                reg.co2_ppm.to_string(),
                reg.ch4_ppm.to_string(),
                reg.created_at.to_rfc3339(),
                reg.tiempo_relativo_min.to_string(),
            ])?;
        }
        writer.flush()?;
        
        total_registros_guardados += registros_diarios.len();
        println!("  ✓ {} → {} registros", fecha, registros_diarios.len());
    }
    
    println!("\n✅ PROCESO COMPLETADO");
    println!("📁 Archivos generados en: {}", output_dir);
    println!("📊 Total registros segmentados: {}", total_registros_guardados);
    
    Ok(())
}