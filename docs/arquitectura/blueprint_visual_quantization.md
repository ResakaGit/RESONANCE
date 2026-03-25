# Módulo: Cuantización Visual por Atención

Blueprint de arquitectura para la integración del LOD Sensorio ($A$) en la tubería de renderizado procedural (GPU).
Fuentes:
- `docs/sprints/README.md` + `src/rendering/quantized_color/` (sprint raíz eliminado)
- `design/VISUAL_QUANTIZATION.md`

## 1) Frontera y Responsabilidad
- **Qué Resuelve**: Optimiza el fragment shader para calcular índices térmicos o de color con menor precisión computacional en los sectores donde la Atención ($A$) es nula o baja.
- **Qué NO Resuelve**: No deforma la geometría (GF2), no altera la iluminación general (GF3).

## 2) Trade-offs y Complejidad (Yanagi)
- **Costo de Sincronización**: Enviar el grid de $A$ a la GPU cada frame puede generar un cuello de botella en el bus PCI-e. 
- **Solución Numérica**: Si $A$ se calcula cada 10 frames en CPU, solo se sube el buffer a la GPU cada 10 frames (`Query<&AttentionGrid, Changed>`). Dentro del shader, si $A < umbral$, reducir los samples del 1D LUT a 1 o calcular flat color en lugar de interpolación.
- **Beneficio Principal**: Cache hits en texturas o LUTs y reducción de ALU instructions. Fundamental para escalar a miles de entidades en la periferia visual.
