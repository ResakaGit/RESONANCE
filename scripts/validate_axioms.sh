#!/usr/bin/env bash
# validate_axioms.sh
#
# Validación RAM-safe de conformidad axiomática por capas.
# Implementa la escalera de docs/design/AXIOM_LAYER_VALIDATION_MATRIX.md §3:
#   Paso 1 — grep de invariantes negativas (Hard Blocks), costo 0.
#   Paso 2 — cargo check --lib (un build compartido).
#   Paso 3 — cargo test --lib <módulo> de math pura (opcional, --deep).
#
# NUNCA corre `cargo test` completo ni activa features gpu/bridge/experimental.
#
# Usage:
#   ./scripts/validate_axioms.sh            # pasos 1-2 (barato, sin correr tests)
#   ./scripts/validate_axioms.sh --deep     # + paso 3 (unit tests de equations)
# Exit 0 si todo pasa, 1 si alguna verificación falla.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"
SRC="$REPO_ROOT/src"
DEEP=0
[ "${1:-}" = "--deep" ] && DEEP=1

fail=0

echo "=== Paso 1: invariantes negativas (grep, sin compilar) ==="

# Hard Block 11 — NO unwrap()/expect()/panic!() en systems (src/simulation/).
hits=$(grep -rn -E '\.(unwrap|expect)\(|panic!\(' "$SRC/simulation" \
    --include='*.rs' | grep -v -E '#\[cfg\(test\)\]|mod tests|// *DEBT' || true)
if [ -n "$hits" ]; then
    echo "WARN: unwrap/expect/panic en systems (revisar, Hard Block 11):"
    echo "$hits" | sed 's/^/  /'
else
    echo "PASS: sin unwrap/expect/panic en src/simulation/"
fi

# Hard Block 7 — NO String en components (layers/). Heurística: campo `: String`.
hits=$(grep -rn -E ':\s*String\b' "$SRC/layers" --include='*.rs' \
    | grep -v -E '#\[cfg\(test\)\]|fn |//' || true)
if [ -n "$hits" ]; then
    echo "WARN: posible String en components (Hard Block 7):"
    echo "$hits" | sed 's/^/  /'
else
    echo "PASS: sin String en campos de src/layers/"
fi

# Derivacional — literales float sospechosos en layers/ fuera de tests/comentarios.
# Señala candidatos a revisión manual contra las 4 constantes fundamentales.
echo "INFO: candidatos a número mágico en src/layers/ (revisar derivación):"
grep -rn -E '=\s*-?[0-9]+\.[0-9]+' "$SRC/layers" --include='*.rs' \
    | grep -v -E '#\[cfg\(test\)\]|assert|//|1\.0|0\.0' \
    | sed 's/^/  /' || echo "  (ninguno)"

echo ""
echo "=== Paso 2: cargo check --lib (un build compartido) ==="
if cargo check --lib --quiet; then
    echo "PASS: cargo check --lib"
else
    echo "FAIL: cargo check --lib"
    fail=1
fi

if [ "$DEEP" = "1" ]; then
    echo ""
    echo "=== Paso 3: unit tests de math pura (--lib, sin runtime/GPU) ==="
    MODULES=(
        "blueprint::equations::conservation"
        "derived_thresholds"
        "blueprint::equations::core_physics"
        "simulation::species_to_qe"
        "simulation::autopoiesis_bridge"
    )
    for m in "${MODULES[@]}"; do
        if cargo test --lib "$m" --quiet -- --test-threads=1 >/dev/null 2>&1; then
            echo "PASS: cargo test --lib $m"
        else
            echo "FAIL: cargo test --lib $m"
            fail=1
        fi
    done
fi

echo ""
echo "=== Resumen ==="
if [ "$fail" -gt 0 ]; then
    echo "Validación axiomática: FALLÓ (ver arriba)."
    exit 1
else
    echo "Validación axiomática: OK (RAM-safe, sin suite completa)."
    exit 0
fi
