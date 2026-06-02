## Plan: TUI para selección de tópico y resumen

Agregar una interfaz TUI estilo “neofetch” (animación a la izquierda, menú/info a la derecha) para que el usuario elija un tópico (subcarpeta inmediata dentro de `docs/`) a resumir, o salir. La UI debe mantener una estética retro inspirada en Rust y mostrar una animación ASCII de un cangrejo abriendo/cerrando pinzas. Se recomienda `ratatui` (fork activo del crate `tui`) + `crossterm` (compatible con Windows). La lógica de resumen existente permanece en `Summarizer`; la TUI solo orquesta selección y presentación, respetando SOLID.

**Pasos**
1. Descubrimiento de tópicos (subcarpetas)
   1) Extender el puerto de filesystem para soportar listado de subdirectorios: en `src/ports.rs`, agregar a `FileSystem` un método `list_dirs(dir: &Path) -> Result<Vec<PathBuf>, io::Error>` (o nombre equivalente). *Depende de 1.2.*
   2) Implementar el listado en el adaptador estándar: en `src/input.rs`, agregar una función pura `list_dirs(dir: &Path) -> Result<Vec<PathBuf>, io::Error>` que devuelva solo entradas inmediatas que sean directorio (1 nivel), y hacer que `StdFileSystem` implemente el nuevo método usando esa función.
   3) En la capa de UI (no en el core), convertir `Vec<PathBuf>` a una lista de “tópicos” mostrando el nombre de carpeta (`file_name`). Ordenar alfabéticamente para UX estable.
   4) Decidir comportamiento cuando no hay subcarpetas en `docs/`: mostrar un estado vacío con mensaje y opción “Salir” (mínimo). (Opcional de compatibilidad: ofrecer un tópico “(raíz)” para resumir `.txt` en `docs/` si existen).

2. Dependencias y límites de arquitectura
   1) Agregar dependencias en `Cargo.toml`: `ratatui` y `crossterm`. Mantener el resto del core sin dependencias TUI.
   2) Mantener `Summarizer` y módulos de procesamiento sin referencias a `ratatui`/`crossterm` (DIP): la UI vive en un módulo nuevo y se invoca desde `main.rs`.

3. Nuevo módulo de UI (TUI)
   1) Crear un módulo `src/ui/` con estas responsabilidades:
      - `ui::app` (estado + bucle de eventos): estado de pantalla actual, selección, tick de animación, y ejecución del resumen en background.
      - `ui::render` (dibujo): funciones puras que rendericen cada pantalla en base al estado.
      - `ui::crab` (frames ASCII): 2+ frames (pinzas abiertas/cerradas) y función para obtener el frame actual.
      - `ui::model` (si hace falta): `TopicItem { name, path }`, `MenuItem` y enums de pantalla.
   2) Diseño de pantallas (mínimo, extensible):
      - Pantalla “Selector”:
        - Columna izquierda: animación ASCII del cangrejo (cambia de frame con un tick cada ~100–200ms).
        - Columna derecha: título/bloque retro y un menú con: lista de tópicos + una opción “Salir”. Indicador de teclas: ↑/↓ para navegar, Enter para seleccionar, `q` para salir.
      - Pantalla “Procesando…”:
        - Mantener animación izquierda.
        - Derecha: texto “Procesando {tópico}…” y spinner simple (opcional) para indicar trabajo.
      - Pantalla “Resumen”:
        - Mantener animación izquierda.
        - Derecha: mostrar resumen (top N oraciones) en formato legible (una oración por línea, con prefijo numérico). Mostrar warnings (si existen) de forma compacta.
        - Opciones mínimas: `Esc` o `b` para volver al selector, `q` para salir. (Esto cumple “mostrar resumen” y permite resumir otro tópico sin reiniciar.)
   3) Layout tipo neofetch:
      - Usar `ratatui::layout::Layout` horizontal con un panel izquierdo de ancho fijo (según el ancho del ASCII) y panel derecho flexible.
      - Usar `Block` con bordes y estilo retro (por ejemplo: bordes dobles si están disponibles, títulos cortos, y colores/atributos sobrios). Evitar hardcode excesivo; priorizar `Modifier::BOLD` y un color de acento consistente.

4. Ejecución del resumen desde la UI (sin congelar animación)
   1) Al seleccionar un tópico, disparar un thread de trabajo que llame a `Summarizer::summarize_dir(&topic_path, top_n)`.
   2) Comunicar resultado a la UI mediante canal `std::sync::mpsc` (o similar). Mientras tanto, el loop de UI sigue recibiendo eventos + tick de animación.
   3) Cuando llega `SummaryReport`, pasar a pantalla “Resumen”. Si llega error, mostrar pantalla/estado de error simple con opción de volver.

5. Integración en el binario
   1) Actualizar `src/lib.rs` para exportar el nuevo módulo `ui` si se necesita desde `main.rs`.
   2) Reemplazar el flujo actual en `src/main.rs`:
      - Crear `StdFileSystem`, `DefaultTokenizer`, `TfidfRanker`, y `Summarizer` como hoy.
      - En vez de llamar directo a `summarize_dir`, iniciar la TUI `ui::run(...)` pasando: `docs_dir` (por defecto `docs`), el `FileSystem` (o `StdFileSystem`) para listar tópicos y el `Summarizer` para ejecutar resúmenes.
      - Mantener el argumento actual `env::args().nth(1)` como base dir (ahora representa el “directorio de tópicos”).

6. Tests y verificaciones
   1) Unit tests de listado de tópicos: agregar tests en `tests/input.rs` para `list_dirs` (creando subcarpetas temporales y validando que solo devuelve directorios inmediatos).
   2) Mantener tests existentes de tokenizer/processor.
   3) Verificación manual:
      - Crear `docs/{topic1,topic2}/` con `.txt`.
      - Ejecutar `cargo run` y validar: selector muestra tópicos, animación corre, Enter resume el tópico, se muestra resumen, `Esc` vuelve, `q` sale.
      - Probar terminal chico: asegurar que la UI no panic (degradar: mostrar mensaje “terminal muy pequeño”).

**Archivos relevantes**
- src/main.rs — punto de entrada: reemplazar salida por consola por invocación a la TUI.
- src/ports.rs — extender `FileSystem` con listado de directorios para tópicos.
- src/input.rs — implementar `list_dirs` + `StdFileSystem::list_dirs`.
- src/summarizer.rs — no debería requerir cambios (se reutiliza `summarize_dir` pasando la carpeta del tópico).
- Cargo.toml — agregar `ratatui` y `crossterm`.
- src/lib.rs — exportar `pub mod ui;` si la TUI vive en la lib.
- (nuevo) src/ui/mod.rs, src/ui/app.rs, src/ui/render.rs, src/ui/crab.rs — UI y animación.
- tests/input.rs — nuevos tests para listado de tópicos.

**Verificación**
1. `cargo test` (asegura que el agregado a `FileSystem` compila y que `list_dirs` funciona).
2. `cargo run` y navegación completa (selector → procesando → resumen → volver → salir).
3. Confirmar que en Windows se restaura el terminal al salir (raw mode off, cursor visible, salir de alternate screen).

**Decisiones**
- Tópicos: subcarpetas inmediatas de `docs/`, no recursivo.
- Post-resumen: volver al selector para resumir otro.
- Librería: usar `ratatui` (API estilo `tui`) por estabilidad, con backend `crossterm`.

**Further Considerations**
1. Compatibilidad con el modo anterior (sin subcarpetas): elegir entre “estado vacío” (mínimo) o “tópico raíz” (migración suave). Recomendación: “tópico raíz” solo si existe demanda; si no, mantener mínimo.
2. Resumen largo: por ahora truncar a altura visible; si más adelante se necesita, agregar scroll (manteniendo el mismo layout).
