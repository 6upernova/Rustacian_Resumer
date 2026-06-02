# Plan de Implementación: Resumidor Extractivo TF-IDF en Rust

## Objetivo

Aplicación de consola en Rust que lee archivos `.txt` de un directorio, los procesa
concurrentemente (un hilo por documento) y produce un resumen extractivo basado en
el algoritmo TF-IDF, seleccionando hasta 10 oraciones de mayor puntaje.

---

## 1. Estructura del Proyecto

```
resumidor-rust/
├── Cargo.toml
├── Cargo.lock
├── docs/                       # Archivos .txt de entrada (Wikipedia)
│   ├── articulo1.txt
│   ├── articulo2.txt
│   └── ...
└── src/
    ├── main.rs                 # Punto de entrada, orquestación y CLI
    ├── input.rs                # Lectura del sistema de archivos
    ├── tokenizer.rs            # Segmentación y tokenización
    ├── processor.rs            # Cálculo TF-IDF y selección de oraciones
    └── output.rs               # Formateo e impresión de resultados
```

---

## 2. Dependencias (`Cargo.toml`)

```toml
[package]
name    = "resumidor-rust"
version = "0.1.0"
edition = "2021"

[dependencies]
# Sin dependencias externas obligatorias.
# Toda la funcionalidad se implementa con la biblioteca estándar de Rust.
# Opcionales (solo si el agente las considera necesarias):
#   regex = "1"          # Para segmentación de oraciones más robusta
#   unicode-segmentation = "1"  # Para tokenización Unicode correcta
```

> **Nota para el agente:** preferir la biblioteca estándar. Agregar crates externos
> solo si la implementación manual resulta excesivamente compleja o incorrecta.

---

## 3. Tipos de Datos Compartidos

Definir en `main.rs` o en un módulo `types.rs` los tipos que cruzan módulos:

```rust
/// Representa un documento leído del disco.
pub struct Document {
    pub path: PathBuf,       // Ruta original del archivo
    pub content: String,     // Contenido crudo
}

/// Oración tokenizada lista para ser puntuada.
pub struct ProcessedSentence {
    pub raw: String,          // Texto original (sin modificar)
    pub tokens: Vec<String>,  // Tokens limpios (minúsc., sin puntuación, sin stopwords)
    pub source: PathBuf,      // Documento de origen
}

/// Oración con su puntaje TF-IDF final.
pub struct ScoredSentence {
    pub raw: String,
    pub score: f64,
    pub source: PathBuf,
}
```

---

## 4. Módulo `input.rs` — Entrada

**Responsabilidad:** interactuar con el sistema de archivos. No conoce nada de
tokenización ni de TF-IDF.

### Funciones a implementar

```rust
/// Recorre `dir` y devuelve los paths de todos los archivos con extensión `.txt`.
/// Error si `dir` no existe o no es un directorio.
pub fn list_txt_files(dir: &Path) -> Result<Vec<PathBuf>, io::Error>

/// Lee el contenido de `path` como UTF-8.
/// Error si el archivo no se puede leer.
pub fn read_file(path: &Path) -> Result<String, io::Error>
```

### Comportamiento esperado

- `list_txt_files` usa `std::fs::read_dir` y filtra por extensión `.txt`
  (comparación insensible a mayúsculas: `.TXT` también es válido).
- `read_file` usa `std::fs::read_to_string`.
- Ambas funciones propagan errores; **no** llaman a `eprintln!` ni a `process::exit`.
- El orden de los archivos devueltos por `list_txt_files` no necesita ser determinista.

---

## 5. Módulo `tokenizer.rs` — Tokenización

**Responsabilidad:** transformar texto crudo en tokens y oraciones. No toca el disco
ni produce salida.

### Stopwords

Definir una constante `STOPWORDS: &[&str]` con al menos las siguientes palabras
en inglés (idioma de los artículos Wikipedia):

```
a, an, the, and, or, but, in, on, at, to, for, of, with, by, from,
is, are, was, were, be, been, being, have, has, had, do, does, did,
will, would, could, should, may, might, shall, that, this, these,
those, it, its, as, if, not, no, nor, so, yet, both, either, just,
than, then, such, when, which, who, whom, while, where, how, what,
```

> Extender la lista si se considera necesario para mejorar la calidad del resumen.

### Funciones a implementar

```rust
/// Divide `text` en oraciones usando como delimitadores: `.`, `!`, `?`
/// seguidos de espacio o fin de string, y saltos de línea dobles (`\n\n`).
/// Descarta oraciones vacías o con menos de 5 tokens (antes del filtrado).
/// Conserva el texto original (sin modificar) en cada entrada devuelta.
pub fn split_sentences(text: &str) -> Vec<String>

/// Convierte una oración en tokens:
///   1. Convertir a minúsculas.
///   2. Reemplazar puntuación (todo carácter no alfanumérico excepto apóstrofe)
///      con espacio.
///   3. Dividir por espacios en blanco.
///   4. Descartar tokens vacíos y stopwords.
/// Devuelve Vec vacío si no quedan tokens.
pub fn tokenize(sentence: &str) -> Vec<String>
```

### Notas de implementación

- Usar `char::is_alphanumeric()` y `char::is_ascii_punctuation()` para clasificar
  caracteres.
- La segmentación de oraciones con expresiones regulares es aceptable si se importa
  el crate `regex`; de lo contrario, implementar manualmente con `split` y heurísticas.
- **No** modificar el campo `raw` de `ProcessedSentence`; la oración original debe
  conservarse tal cual para la salida final.

---

## 6. Módulo `processor.rs` — Procesamiento TF-IDF

**Responsabilidad:** implementar el algoritmo TF-IDF y seleccionar las mejores
oraciones. No toca el disco ni la terminal.

### Funciones a implementar

```rust
/// Dado el conjunto global de oraciones procesadas, calcula el IDF de cada token.
///
/// IDF(t) = ln( N / (1 + df(t)) )
///
/// donde:
///   N     = número total de oraciones (en todos los documentos)
///   df(t) = número de oraciones que contienen al menos una vez el token t
///
/// Devuelve un HashMap<String, f64> con el IDF de cada token que aparece
/// al menos una vez en el corpus.
pub fn compute_idf(sentences: &[ProcessedSentence]) -> HashMap<String, f64>

/// Calcula el puntaje TF-IDF de una oración:
///
///   score(s) = Σ  TF(t, s) × IDF(t)
///              t ∈ tokens(s)
///
/// donde TF(t, s) = (ocurrencias de t en s) / (total de tokens en s)
///
/// Si la oración no tiene tokens, su puntaje es 0.0.
pub fn score_sentence(sentence: &ProcessedSentence, idf: &HashMap<String, f64>) -> f64

/// Puntúa todas las oraciones y devuelve las `top_n` de mayor puntaje,
/// ordenadas de mayor a menor puntaje.
/// Si hay empate en el puntaje, se preserva el orden de aparición original.
pub fn select_top_sentences(
    sentences: Vec<ProcessedSentence>,
    idf: &HashMap<String, f64>,
    top_n: usize,
) -> Vec<ScoredSentence>
```

### Notas de implementación

- Usar `f64::ln` para el logaritmo natural.
- Para calcular `df(t)`, iterar sobre todas las oraciones y usar un `HashSet`
  por oración para no contar duplicados dentro de la misma oración.
- En `select_top_sentences`, usar `sort_unstable_by` con `f64::total_cmp` (Rust ≥ 1.62)
  o `partial_cmp().unwrap_or(Ordering::Equal)`.
- El valor `top_n` debe recibirse como parámetro (el llamador pasa `10`).

---

## 7. Módulo `output.rs` — Salida

**Responsabilidad:** formatear e imprimir resultados. No implementa lógica de negocio.

### Funciones a implementar

```rust
/// Imprime el encabezado de la aplicación.
pub fn print_header(dir: &Path, file_count: usize)

/// Imprime el resumen final: las oraciones seleccionadas con su puntaje y fuente.
///
/// Formato sugerido:
///
///   === RESUMEN EXTRACTIVO ===
///   [1] (score: 3.4521) [fuente: articulo1.txt]
///       The mitochondria is the powerhouse of the cell ...
///
///   [2] (score: 3.1200) [fuente: articulo2.txt]
///       ...
///
pub fn print_summary(sentences: &[ScoredSentence])

/// Imprime un mensaje de error a stderr y termina el proceso con código 1.
pub fn fatal_error(msg: &str) -> !
```

### Notas de implementación

- `print_summary` escribe a **stdout** con `println!`.
- `fatal_error` escribe a **stderr** con `eprintln!` y llama a `std::process::exit(1)`.
- Truncar las oraciones largas a 200 caracteres en la salida si se desea legibilidad;
  de lo contrario, imprimir completas.

---

## 8. Módulo `main.rs` — Orquestación y Concurrencia

**Responsabilidad:** coordinar todos los módulos, gestionar hilos y el flujo de datos.
No implementa lógica de algoritmos.

### Flujo completo

```
main()
 │
 ├─ 1. Leer argumento CLI: directorio de entrada (default: "docs/")
 │
 ├─ 2. input::list_txt_files(dir)  →  Vec<PathBuf>
 │       └─ Si error o lista vacía → output::fatal_error(...)
 │
 ├─ 3. FASE PARALELA — un hilo por documento:
 │       Para cada path en la lista:
 │         spawn(thread) {
 │           content = input::read_file(path)
 │           sentences_raw = tokenizer::split_sentences(&content)
 │           processed = sentences_raw
 │               .iter()
 │               .map(|s| ProcessedSentence {
 │                   raw:    s.clone(),
 │                   tokens: tokenizer::tokenize(s),
 │                   source: path.clone(),
 │               })
 │               .filter(|ps| !ps.tokens.is_empty())
 │               .collect::<Vec<_>>()
 │           Enviar processed por canal (mpsc::Sender)
 │         }
 │
 ├─ 4. Recolectar resultados de todos los hilos
 │       →  all_sentences: Vec<ProcessedSentence>
 │
 ├─ 5. processor::compute_idf(&all_sentences)  →  idf: HashMap<String, f64>
 │
 ├─ 6. processor::select_top_sentences(all_sentences, &idf, 10)
 │       →  top: Vec<ScoredSentence>
 │
 └─ 7. output::print_header(dir, file_count)
        output::print_summary(&top)
```

### Implementación de la concurrencia

Usar **`std::sync::mpsc`** (multiple-producer, single-consumer) y **`std::thread`**:

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel::<Vec<ProcessedSentence>>();
let mut handles = Vec::new();

for path in paths {
    let tx = tx.clone();
    let handle = thread::spawn(move || {
        // leer + segmentar + tokenizar
        // tx.send(processed_sentences).ok();
    });
    handles.push(handle);
}

drop(tx); // cerrar el extremo del sender del hilo principal

// Esperar a todos los hilos
for h in handles {
    h.join().expect("El hilo de procesamiento falló");
}

// Recolectar
let all_sentences: Vec<ProcessedSentence> = rx.into_iter().flatten().collect();
```

### Manejo de errores en hilos

- Si un hilo no puede leer un archivo, imprimir un **warning** a stderr
  (`eprintln!("[WARN] No se pudo leer {:?}: {}", path, e)`) y enviar un
  `Vec` vacío (o no enviar nada). El programa **no** debe abortar por un
  archivo inaccesible.

### Argumento CLI

```
USAGE:
    tfidf_summarizer [DIRECTORIO]

ARGS:
    DIRECTORIO    Ruta al directorio con archivos .txt [default: docs]
```

Leer con `std::env::args().nth(1)` y usar `"docs"` como valor por defecto.

---

## 9. Invariantes y Reglas de Calidad

| # | Regla |
|---|-------|
| 1 | Cada módulo (`input`, `tokenizer`, `processor`, `output`) vive en su propio archivo `.rs` y se declara como `mod` en `main.rs`. |
| 2 | Ningún módulo importa directamente a otro lateral (p.ej. `tokenizer` no importa `input`). Todo el flujo de datos pasa por `main.rs`. |
| 3 | Las funciones públicas de cada módulo **no** realizan I/O no documentada (sin `println!` ocultos dentro de `processor.rs`, etc.). |
| 4 | El código compila sin warnings con `cargo build`. |
| 5 | El código pasa `cargo clippy` sin errores (warnings menores son aceptables). |
| 6 | No se usan `unwrap()` o `expect()` en rutas de error esperadas; sí en `join()` de hilos o invariantes comprobados. |
| 7 | El programa termina con código de salida `0` en caso de éxito y `1` en caso de error fatal. |

---

## 10. Secuencia de Implementación Recomendada

El agente debe implementar en este orden para poder probar incrementalmente:

1. **Scaffolding**: crear `Cargo.toml` y los cinco archivos `.rs` con stubs vacíos
   (funciones que retornan `todo!()`).
2. **`input.rs`**: implementar `list_txt_files` y `read_file`; verificar con `cargo test`.
3. **`tokenizer.rs`**: implementar `split_sentences` y `tokenize`; agregar unit tests
   con cadenas de ejemplo.
4. **`processor.rs`**: implementar `compute_idf`, `score_sentence` y
   `select_top_sentences`; verificar manualmente con datos de prueba pequeños.
5. **`output.rs`**: implementar las tres funciones de salida.
6. **`main.rs`**: conectar todos los módulos con el flujo concurrente descrito en
   la sección 8.
7. **Integración final**: ejecutar con los archivos Wikipedia reales y verificar
   que el resumen sea coherente.

---

## 11. Tests Unitarios

Agregar al final de cada módulo un bloque `#[cfg(test)]` con al menos los siguientes casos:

### `tokenizer.rs`

```rust
#[test]
fn test_tokenize_removes_stopwords() { ... }

#[test]
fn test_tokenize_lowercases() { ... }

#[test]
fn test_split_sentences_basic() { ... }

#[test]
fn test_split_sentences_empty_input() { ... }
```

### `processor.rs`

```rust
#[test]
fn test_idf_single_sentence() { ... }   // N=1, df=1 → ln(1/2) = -0.693...

#[test]
fn test_score_empty_tokens() { ... }    // score debe ser 0.0

#[test]
fn test_select_top_n_returns_correct_count() { ... }

#[test]
fn test_select_top_n_ordered_descending() { ... }
```

---

## 12. Ejemplo de Salida Esperada

```
============================================================
  Resumidor TF-IDF
  Directorio: docs/  |  Archivos procesados: 5
============================================================

=== RESUMEN EXTRACTIVO (top 10 oraciones) ===

[1] score: 4.8823  |  fuente: quantum_mechanics.txt
    In quantum mechanics, the uncertainty principle states that ...

[2] score: 4.6210  |  fuente: general_relativity.txt
    General relativity is a geometric theory of gravitation published by ...

[3] score: 4.4017  |  fuente: quantum_mechanics.txt
    The Copenhagen interpretation is a collection of views about the ...

... (hasta 10 oraciones)
```

---

## 13. Checklist Final para el Agente

- [ ] `cargo build --release` finaliza sin errores ni warnings.
- [ ] `cargo test` pasa todos los tests.
- [ ] `cargo clippy` no reporta errores.
- [ ] Ejecutar con un directorio vacío produce un error claro en stderr y exit code 1.
- [ ] Ejecutar con un directorio con un solo archivo produce hasta 10 oraciones.
- [ ] Ejecutar con 5+ archivos Wikipedia muestra oraciones de distintas fuentes.
- [ ] El tiempo de ejecución con múltiples archivos es menor que con un único hilo
  (verificable con `time` o `std::time::Instant`).
- [ ] La lógica de TF-IDF está completamente en `processor.rs` y no en `main.rs`.
- [ ] El módulo `output.rs` es el **único** lugar con llamadas a `println!`/`eprintln!`
  en rutas normales de ejecución.