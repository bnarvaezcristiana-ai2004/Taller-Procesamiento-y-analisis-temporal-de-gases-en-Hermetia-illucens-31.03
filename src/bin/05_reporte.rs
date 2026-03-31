use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use csv::ReaderBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 05_REPORTE_ANALITICO ===\n");
    
    let input_dir = "data/processed/datos_por_dia";
    let output_path = "reports/analisis_por_dia.md";
    let incidencias_path = "reports/incidencias.md";
    
    let entries = fs::read_dir(input_dir)?;
    let mut archivos: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "csv"))
        .collect();
    
    archivos.sort_by_key(|e| e.file_name());
    
    let mut reporte = File::create(&output_path)?;
    let mut incidencias = File::create(&incidencias_path)?;
    
    writeln!(reporte, "# Análisis Técnico por Día - Hermetia illucens\n")?;
    writeln!(reporte, "## Procesamiento y análisis temporal de gases (CO2 y CH4)\n")?;
    writeln!(reporte, "Fecha de generación: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(reporte, "---\n")?;
    
    writeln!(incidencias, "# Registro de Incidencias\n")?;
    writeln!(incidencias, "Fecha de generación: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(incidencias, "---\n")?;
    
    println!("📝 Generando reporte para {} días...\n", archivos.len());
    
    for entry in &archivos {
        let filename = entry.file_name();
        let fecha_str = filename.to_string_lossy().replace(".csv", "");
        
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
        
        if tiempos.is_empty() {
            writeln!(incidencias, "## {} - SIN DATOS\n", fecha_str)?;
            writeln!(incidencias, "**Incidencia**: Archivo sin registros válidos\n")?;
            continue;
        }
        
        // ✅ CORRECCIÓN: Usar closures en lugar de f64::max directamente
        let max_co2 = co2_vals.iter().fold(0.0f64, |a, &b| f64::max(a, b));
        let min_co2 = co2_vals.iter().fold(f64::MAX, |a, &b| f64::min(a, b));
        let max_ch4 = ch4_vals.iter().fold(0.0f64, |a, &b| f64::max(a, b));
        let min_ch4 = ch4_vals.iter().fold(f64::MAX, |a, &b| f64::min(a, b));
        
        let duracion_min = *tiempos.last().unwrap_or(&0.0);
        
        let (pend_co2, r2_co2) = proyecto_gases_hi::calcular_pendiente_regresion_lineal(&tiempos, &co2_vals);
        let (pend_ch4, r2_ch4) = proyecto_gases_hi::calcular_pendiente_regresion_lineal(&tiempos, &ch4_vals);
        
        writeln!(reporte, "## Análisis del día: {}\n", fecha_str)?;
        writeln!(reporte, "### Resumen estadístico\n")?;
        writeln!(reporte, "- **Duración del monitoreo**: {:.2} minutos", duracion_min)?;
        writeln!(reporte, "- **Total de registros**: {}", tiempos.len())?;
        writeln!(reporte, "\n#### CO2 (ppm)")?;
        writeln!(reporte, "- Máximo: {:.2}", max_co2)?;
        writeln!(reporte, "- Mínimo: {:.2}", min_co2)?;
        writeln!(reporte, "- Rango: {:.2}", max_co2 - min_co2)?;
        writeln!(reporte, "- Pendiente: {:.4} ppm/min", pend_co2)?;
        writeln!(reporte, "- R²: {:.4}", r2_co2)?;
        writeln!(reporte, "\n#### CH4 (ppm)")?;
        writeln!(reporte, "- Máximo: {:.2}", max_ch4)?;
        writeln!(reporte, "- Mínimo: {:.2}", min_ch4)?;
        writeln!(reporte, "- Rango: {:.2}", max_ch4 - min_ch4)?;
        writeln!(reporte, "- Pendiente: {:.4} ppm/min", pend_ch4)?;
        writeln!(reporte, "- R²: {:.4}", r2_ch4)?;
        
        writeln!(reporte, "\n### Interpretación técnica\n")?;
        
        if pend_co2 > 10.0 {
            writeln!(reporte, "- **CO2**: Tendencia alcuenta pronunciada ({:.2} ppm/min). Posible aumento en actividad metabólica de las larvas.", pend_co2)?;
        } else if pend_co2 < -10.0 {
            writeln!(reporte, "- **CO2**: Tendencia decreciente. Posible disminución en actividad o consumo completado.")?;
        } else {
            writeln!(reporte, "- **CO2**: Comportamiento estable o con variaciones moderadas.")?;
        }
        
        if pend_ch4 > 1.0 {
            writeln!(reporte, "- **CH4**: Aumento en producción de metano. Posible relación con dieta o condiciones anaeróbicas.")?;
        } else if pend_ch4 < -1.0 {
            writeln!(reporte, "- **CH4**: Disminución en producción.")?;
        } else {
            writeln!(reporte, "- **CH4**: Producción estable.")?;
        }
        
        if max_co2 > 4000.0 {
            writeln!(reporte, "\n⚠️ **Alerta**: CO2 alcanzó valores altos (>4000 ppm). Verificar ventilación.")?;
        }
        
        writeln!(reporte, "\n### Gráficos disponibles\n")?;
        writeln!(reporte, "- CO2: `figures/diarios/{}_co2.png`", fecha_str)?;
        writeln!(reporte, "- CH4: `figures/diarios/{}_ch4.png`", fecha_str)?;
        writeln!(reporte, "- Conjunto: `figures/diarios/{}_conjunto.png`", fecha_str)?;
        writeln!(reporte, "\n---\n")?;
        
        println!("  ✓ {}", fecha_str);
    }
    
    writeln!(incidencias, "\n## Metodología\n")?;
    writeln!(incidencias, "- Se eliminaron registros con valores vacíos o inválidos")?;
    writeln!(incidencias, "- Se filtraron registros con saturación del sensor (6000 ppm)")?;
    writeln!(incidencias, "- Tiempo relativo calculado desde el primer registro de cada día")?;
    writeln!(incidencias, "- Pendientes calculadas por regresión lineal simple")?;
    
    println!("\n✅ PROCESO COMPLETADO");
    println!("📁 Reportes generados:");
    println!("   - {}", output_path);
    println!("   - {}", incidencias_path);
    
    Ok(())
}