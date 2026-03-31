# Bitácora de Limpieza de Datos

## Fecha de ejecución: 2026-03-31 15:14:48


## Transformaciones aplicadas:

1. **Conservación del archivo original**: data/raw/datosbase.csv
2. **Columnas conservadas**: entry_id, field2 (CO2), field4 (CH4), created_at
3. **Columnas eliminadas**: field1, field3, field5, field6, field7, field8, latitude, longitude, elevation, status
4. **Renombrado**: field2 → co2_ppm, field4 → ch4_ppm
5. **Validación temporal**: Formato RFC3339 verificado
6. **Filtrado de vacíos**: 7 registros eliminados
7. **Filtrado de saturación**: 1744 registros eliminados (valores 6000)

## Resumen:

- Total registros originales: 3850
- Total registros finales: 2106
- Fecha inicio: Some(2025-08-10)
- Fecha fin: Some(2025-09-17)
