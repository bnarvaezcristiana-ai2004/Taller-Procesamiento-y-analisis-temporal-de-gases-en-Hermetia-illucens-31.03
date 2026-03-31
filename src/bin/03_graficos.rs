use proyecto_gases_hi::RegistroDiario;
use plotters::prelude::*;
use std::error::Error;
use std::fs;
use std::path::Path;
use csv::ReaderBuilder;

fn cargar_dia(fecha: &str) -> Result<Vec<RegistroDiario>, Box<dyn Error>> {
    let path = format!("data/processed/datos_por_dia/{}.csv", fecha);
    let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?;
    
    let mut registros = Vec::new();
    for result in reader.records() {
        let record = result?;
        registros.push(RegistroDiario {
            entry_id: record.get(0).unwrap_or("0").parse().unwrap_or(0),
            co2_ppm: record.get(1).unwrap_or("0").parse().unwrap_or(0.0),
            ch4_ppm: record.get(2).unwrap_or("0").parse().unwrap_or(0.0),
            created_at: proyecto_gases_hi::parsear_fecha(record.get(3).unwrap_or(""))?,
            tiempo_relativo_min: record.get(4).unwrap_or("0").parse().unwrap_or(0.0),
        });
    }
    Ok(registros)
}

fn crear_grafico_diario(
    fecha: &str,
    registros: &[RegistroDiario],
    variable: &str,
) -> Result<(), Box<dyn Error>> {
    // ✅ CORRECCIÓN: Guardar el string en una variable primero
    let filename = format!("figures/diarios/{}_{}.png", fecha, variable.to_lowercase());
    
    let root = BitMapBackend::new(&filename, (1200, 600))
        .into_drawing_area();
    
    root.fill(&WHITE)?;
    
    let datos: Vec<(f64, f64)> = registros
        .iter()
        .map(|r| {
            let val = if variable == "CO2" { r.co2_ppm } else { r.ch4_ppm };
            (r.tiempo_relativo_min, val)
        })
        .collect();
    
    if datos.is_empty() {
        return Ok(());
    }
    
    let max_tiempo = datos.iter().map(|(x, _)| *x).fold(0.0f64, f64::max).ceil();
    let max_valor = datos.iter().map(|(_, y)| *y).fold(0.0f64, f64::max);
    let max_valor = (max_valor * 1.1).ceil();
    
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} - {} (ppm) vs Tiempo (min)", fecha, variable),
            ("sans-serif", 25).into_font(),
        )
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..max_tiempo, 0.0..max_valor)?;
    
    chart.configure_mesh().draw()?;
    
    let color = if variable == "CO2" { &BLUE } else { &RED };
    
    chart
        .draw_series(LineSeries::new(datos.clone(), color.stroke_width(2)))?
        .label(variable)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)));
    
    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;
    
    root.present()?;
    
    Ok(())
}

fn crear_grafico_conjunto(
    fecha: &str,
    registros: &[RegistroDiario],
) -> Result<(), Box<dyn Error>> {
    // ✅ CORRECCIÓN: Guardar el string en una variable primero
    let filename = format!("figures/diarios/{}_conjunto.png", fecha);
    
    let root = BitMapBackend::new(&filename, (1200, 800))
        .into_drawing_area();
    
    root.fill(&WHITE)?;
    
    let datos_co2: Vec<(f64, f64)> = registros
        .iter()
        .map(|r| (r.tiempo_relativo_min, r.co2_ppm))
        .collect();
    
    let datos_ch4: Vec<(f64, f64)> = registros
        .iter()
        .map(|r| (r.tiempo_relativo_min, r.ch4_ppm))
        .collect();
    
    if datos_co2.is_empty() && datos_ch4.is_empty() {
        return Ok(());
    }
    
    let max_tiempo = registros.iter().map(|r| r.tiempo_relativo_min).fold(0.0f64, f64::max).ceil();
    
    let max_co2 = datos_co2.iter().map(|(_, y)| *y).fold(0.0f64, f64::max);
    let max_ch4 = datos_ch4.iter().map(|(_, y)| *y).fold(0.0f64, f64::max);
    let max_valor = (max_co2.max(max_ch4) * 1.1).ceil();
    
    let titulo = format!("{} - CO2 y CH4 (ppm) vs Tiempo (min)", fecha);
    
    let mut chart = ChartBuilder::on(&root)
        .caption(titulo.as_str(), ("sans-serif", 25).into_font())
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..max_tiempo, 0.0..max_valor)?;
    
    chart.configure_mesh().draw()?;
    
    chart.draw_series(LineSeries::new(datos_co2, BLUE.stroke_width(2)))?
        .label("CO2")
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.stroke_width(2)));
    
    chart.draw_series(LineSeries::new(datos_ch4, RED.stroke_width(2)))?
        .label("CH4")
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.stroke_width(2)));
    
    chart.configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;
    
    root.present()?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== 03_GENERACION_DE_GRAFICOS ===\n");
    
    let input_dir = "data/processed/datos_por_dia";
    let output_dir = "figures/diarios";
    
    if !Path::new(output_dir).exists() {
        fs::create_dir_all(output_dir)?;
    }
    
    let entries = fs::read_dir(input_dir)?;
    let mut archivos: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "csv"))
        .collect();
    
    archivos.sort_by_key(|e| e.file_name());
    
    println!("📊 Generando gráficos para {} días...\n", archivos.len());
    
    for entry in &archivos {
        let filename = entry.file_name();
        let fecha_str = filename.to_string_lossy().replace(".csv", "");
        
        println!("📈 Procesando: {}", fecha_str);
        
        match cargar_dia(&fecha_str) {
            Ok(registros) => {
                if registros.is_empty() {
                    println!("  ⚠ Sin datos suficientes");
                    continue;
                }
                
                crear_grafico_diario(&fecha_str, &registros, "CO2")?;
                println!("  ✓ CO2 gráfico generado");
                
                crear_grafico_diario(&fecha_str, &registros, "CH4")?;
                println!("  ✓ CH4 gráfico generado");
                
                crear_grafico_conjunto(&fecha_str, &registros)?;
                println!("  ✓ Conjunto gráfico generado");
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
            }
        }
        println!();
    }
    
    println!("✅ PROCESO COMPLETADO");
    println!("📁 Gráficos guardados en: {}", output_dir);
    
    Ok(())
}