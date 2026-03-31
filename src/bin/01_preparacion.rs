use proyecto_gases_hi::{
    parsear_fecha, extraer_fecha, RegistroProcesado,
    filtrar_saturacion,
};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use csv::{ReaderBuilder, WriterBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 01_PREPARACION_DE_DATOS ===\n");
    
    let input_path = "data/raw/datosbase.csv";
    let output_interim = "data/interim/datos_filtrados_ppm.csv";
    let output_processed = "data/processed/datos_limpios.csv";
    let bitacora_path = "reports/bitacora_limpieza.md";
    
    println!("📖 Leyendo archivo: {}", input_path);
    
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(input_path)?;
    
    let mut registros_procesados: Vec<RegistroProcesado> = Vec::new();
    let mut total_registros = 0;
    let mut registros_eliminados_vacios = 0;
    let mut registros_eliminados_saturacion = 0;
    
    println!("🔄 Procesando registros...");
    
    for result in reader.records() {
        let record = match result {
            Ok(r) => r,
            Err(_) => {
                registros_eliminados_vacios += 1;
                continue;
            }
        };
        
        // ✅ CORRECCIÓN: Índices correctos según la estructura del CSV
        // Columna 0: created_at, Columna 1: entry_id, Columna 3: field2 (CO2), Columna 5: field4 (CH4)
        
        if record.len() < 6 {
            registros_eliminados_vacios += 1;
            continue;
        }
        
        let entry_id: u32 = match record.get(1).unwrap_or("").trim().parse() {
            Ok(id) => id,
            Err(_) => {
                registros_eliminados_vacios += 1;
                continue;
            }
        };
        
        let field2_str = record.get(3).unwrap_or("").trim();  // ✅ field2 está en columna 3
        let field4_str = record.get(5).unwrap_or("").trim();  // ✅ field4 está en columna 5
        let created_at_str = record.get(0).unwrap_or("").trim();  // ✅ created_at está en columna 0
        
        if field2_str.is_empty() || field4_str.is_empty() || created_at_str.is_empty() {
            registros_eliminados_vacios += 1;
            continue;
        }
        
        let co2_ppm: f64 = match field2_str.parse() {
            Ok(val) => val,
            Err(_) => {
                registros_eliminados_vacios += 1;
                continue;
            }
        };
        
        let ch4_ppm: f64 = match field4_str.parse() {
            Ok(val) => val,
            Err(_) => {
                registros_eliminados_vacios += 1;
                continue;
            }
        };
        
        let created_at = match parsear_fecha(created_at_str) {
            Ok(dt) => dt,
            Err(_) => {
                registros_eliminados_vacios += 1;
                continue;
            }
        };
        
        let fecha = extraer_fecha(created_at);
        
        registros_procesados.push(RegistroProcesado {
            entry_id,
            co2_ppm,
            ch4_ppm,
            created_at,
            fecha,
        });
        
        total_registros += 1;
    }
    
    println!("✓ Total registros leídos: {}", total_registros);
    println!("✗ Eliminados (vacíos/inválidos): {}", registros_eliminados_vacios);
    
    let antes_saturacion = registros_procesados.len();
    registros_procesados = filtrar_saturacion(registros_procesados);
    registros_eliminados_saturacion = antes_saturacion - registros_procesados.len();
    println!("✗ Eliminados (saturación 6000): {}", registros_eliminados_saturacion);
    
    registros_procesados.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    
    println!("\n💾 Guardando archivo interim...");
    let mut writer_interim = WriterBuilder::new()
        .has_headers(true)
        .from_path(output_interim)?;
    
    writer_interim.write_record(&["entry_id", "co2_ppm", "ch4_ppm", "created_at", "fecha"])?;
    for reg in &registros_procesados {
        writer_interim.write_record(&[
            reg.entry_id.to_string(),
            reg.co2_ppm.to_string(),
            reg.ch4_ppm.to_string(),
            reg.created_at.to_rfc3339(),
            reg.fecha.format("%Y-%m-%d").to_string(),
        ])?;
    }
    writer_interim.flush()?;
    
    println!("💾 Guardando archivo processed...");
    let mut writer_processed = WriterBuilder::new()
        .has_headers(true)
        .from_path(output_processed)?;
    
    writer_processed.write_record(&["entry_id", "co2_ppm", "ch4_ppm", "created_at", "fecha"])?;
    for reg in &registros_procesados {
        writer_processed.write_record(&[
            reg.entry_id.to_string(),
            reg.co2_ppm.to_string(),
            reg.ch4_ppm.to_string(),
            reg.created_at.to_rfc3339(),
            reg.fecha.format("%Y-%m-%d").to_string(),
        ])?;
    }
    writer_processed.flush()?;
    
    println!("\n📝 Generando bitácora...");
    let mut bitacora = File::create(bitacora_path)?;
    writeln!(bitacora, "# Bitácora de Limpieza de Datos\n")?;
    writeln!(bitacora, "## Fecha de ejecución: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(bitacora, "\n## Transformaciones aplicadas:\n")?;
    writeln!(bitacora, "1. **Conservación del archivo original**: {}", input_path)?;
    writeln!(bitacora, "2. **Columnas conservadas**: entry_id, field2 (CO2), field4 (CH4), created_at")?;
    writeln!(bitacora, "3. **Columnas eliminadas**: field1, field3, field5, field6, field7, field8, latitude, longitude, elevation, status")?;
    writeln!(bitacora, "4. **Renombrado**: field2 → co2_ppm, field4 → ch4_ppm")?;
    writeln!(bitacora, "5. **Validación temporal**: Formato RFC3339 verificado")?;
    writeln!(bitacora, "6. **Filtrado de vacíos**: {} registros eliminados", registros_eliminados_vacios)?;
    writeln!(bitacora, "7. **Filtrado de saturación**: {} registros eliminados (valores 6000)", registros_eliminados_saturacion)?;
    writeln!(bitacora, "\n## Resumen:\n")?;
    writeln!(bitacora, "- Total registros originales: {}", total_registros)?;
    writeln!(bitacora, "- Total registros finales: {}", registros_procesados.len())?;
    if !registros_procesados.is_empty() {
        writeln!(bitacora, "- Fecha inicio: {:?}", registros_procesados.first().map(|r| r.fecha))?;
        writeln!(bitacora, "- Fecha fin: {:?}", registros_procesados.last().map(|r| r.fecha))?;
    }
    
    println!("\n✅ PROCESO COMPLETADO EXITOSAMENTE");
    println!("📊 Registros finales: {}", registros_procesados.len());
    println!("📁 Archivos generados:");
    println!("   - {}", output_interim);
    println!("   - {}", output_processed);
    println!("   - {}", bitacora_path);
    
    Ok(())
}