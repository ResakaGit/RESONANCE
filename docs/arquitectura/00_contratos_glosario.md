# Contratos, Glosario y Template

Base de interpretación para todos los blueprints de `docs/arquitectura`.
Referencia conceptual: `docs/design/BLUEPRINT.md` y `docs/design/V6.md`.

## Glosario mínimo

- **Contrato**: interfaz observable del módulo (tipos públicos, eventos, recursos, side-effects).
- **Comportamiento**: cómo evoluciona estado en runtime (orden, condiciones, fallback).
- **Implementación**: decisiones internas (algoritmos, estructuras, trade-offs).
- **Invariante**: condición que siempre debe mantenerse para que el sistema sea válido.
- **Atomicidad de módulo**: grado en el que el módulo mantiene una responsabilidad única.

## Template obligatorio por módulo

Copiar este bloque para cada módulo documentado:

```md
## <Módulo>

### 1) Propósito y frontera
- Qué resuelve.
- Qué no resuelve (límite explícito).

### 2) Superficie pública (contrato)
- Tipos exportados.
- Funciones/sistemas/plugins públicos.
- Eventos y resources leídos/escritos.

### 3) Invariantes y precondiciones
- Invariantes de datos.
- Precondiciones de ejecución.

### 4) Comportamiento runtime
- Fase/schedule en que corre.
- Orden relativo con otros módulos.
- Side-effects y determinismo.

### 5) Implementación y trade-offs
- Estrategia técnica elegida.
- Costo vs valor.
- Límites conocidos.

### 6) Fallas y observabilidad
- Modos de falla esperados.
- Señales/telemetría para detectar desvíos.

### 7) Checklist de atomicidad
- ¿Una responsabilidad principal?
- ¿Acopla más de un dominio?
- ¿Debería dividirse? ¿Por qué?

### 8) Referencias cruzadas
- `docs/design/...`
- `docs/sprints/...`
```

## Niveles de severidad al auditar contratos

- **Alta**: rompe determinismo, orden de fases o invariantes físicos.
- **Media**: ambigüedad de ownership de estado o side-effects implícitos.
- **Baja**: deuda de legibilidad, nombres, o falta de explicitud documental.

## Criterio de “documentar vale la pena”

Documentar como módulo atómico cuando cumple al menos uno:

- Define contrato público consumido por más de un dominio.
- Participa del pipeline crítico de `FixedUpdate`.
- Encapsula ecuaciones/reglas de dominio.
- Introduce compatibilidad o bridge entre subsistemas (ej. 2D/3D).
