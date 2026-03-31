use proyecto_gases_hi::{calcular_pendiente_regresion_lineal, PendienteDia};
use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::fs;
use std::path::Path;
use chrono::NaiveDate;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 04_CALCULO_DE_PENDIENTES ===\n");
    
    let input_dir = "data/processed/datos_por_dia";
    let output_path = "data/processed/resumen_pendientes.csv";
    
    let entries = fs::read_dir(input_dir)?;
    let mut archivos: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "csv"))
        .collect();
    
    archivos.sort_by_key(|e| e.file_name());
    
    let mut pendientes: Vec<PendienteDia> = Vec::new();
    
    println!("📊 Calculando pendientes para {} días...\n", archivos.len());
    
    for entry in &archivos {
        let filename = entry.file_name();
        let fecha_str = filename.to_string_lossy().replace(".csv", "");
        
        let fecha: NaiveDate = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d")?;
        
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(entry.path())?;
        
        let mut tiempos: Vec<f64> = Vec::new();
        let mut co2_vals: Vec<f64> = Vec::new();
        let mut ch4_vals: Vec<f64> = Vec::new();
        
        for result in reader.records() {
            let record = result?;
            let tiempo: f64 = record.get(4).unwrap_or("0").parse().unwrap_or(0.0);
            let co2: f64 = record.get(1).unwrap_or("0").parse().unwrap_or(0.0);
            let ch4: f64 = record.get(2).unwrap_or("0").parse().unwrap_or(0.0);
            
            tiempos.push(tiempo);
            co2_vals.push(co2);
            ch4_vals.push(ch4);
        }
        
        let (pend_co2, r2_co2) = calcular_pendiente_regresion_lineal(&tiempos, &co2_vals);
        let (pend_ch4, r2_ch4) = calcular_pendiente_regresion_lineal(&tiempos, &ch4_vals);
        
        pendientes.push(PendienteDia {
            fecha,
            pendiente_co2: pend_co2,
            pendiente_ch4: pend_ch4,
            r2_co2,
            r2_ch4,
        });
        
        println!("{}:", fecha_str);
        println!("  CO2: pendiente = {:.4} ppm/min, R² = {:.4}", pend_co2, r2_co2);
        println!("  CH4: pendiente = {:.4} ppm/min, R² = {:.4}", pend_ch4, r2_ch4);
        println!();
    }
    
    println!("💾 Guardando resumen...");
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .from_path(output_path)?;
    
    writer.write_record(&["fecha", "pendiente_co2", "r2_co2", "pendiente_ch4", "r2_ch4"])?;
    for p in &pendientes {
        writer.write_record(&[
            p.fecha.format("%Y-%m-%d").to_string(),
            p.pendiente_co2.to_string(),
            p.r2_co2.to_string(),
            p.pendiente_ch4.to_string(),
            p.r2_ch4.to_string(),
        ])?;
    }
    writer.flush()?;
    
    println!("\n✅ PROCESO COMPLETADO");
    println!("📁 Resumen guardado en: {}", output_path);
    
    Ok(())
}