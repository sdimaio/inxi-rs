# Bootstrap del Progetto Rust

## Stato attuale

E' stato avviato un workspace Rust separato in:

- `inxi-rs/`

Struttura iniziale:

- `crates/inxi-core`
- `crates/inxi-cli`

## Decisioni fissate

- il sorgente Perl originale resta intatto
- il clone Rust vive in una sottodirectory dedicata
- l'architettura parte come `core` riusabile + `cli` separata
- la TUI futura non fa parte della v1, ma il design la rende possibile

## Cosa e' gia' implementato

### `inxi-core`

- modello dati del report
- `CapabilityReport`
- planner iniziale
- collector Linux per:
  - `System`
  - `Machine`
  - `CPU`
  - `Memory`
  - `Graphics`
  - `Network`
  - `Drives`
  - `Partitions`
  - `Swap`
  - `Info`
- renderer `screen`
- renderer `json`

### `inxi-cli`

- parsing CLI iniziale con:
  - `-b`
  - `-S`
  - `-M`
  - `-C`
  - `-m`
  - `-G`
  - `-N`
  - `-D`
  - `-P`
  - `-j`
  - `-I`
  - `-x`, `-xx`
  - `-a`
  - `-z`, `-Z`
  - `--output screen|json`

## Stato qualitativo

Verifiche eseguite:

- `cargo fmt --all`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo run -q -p inxi-cli -- -b`
- `cargo run -q -p inxi-cli -- -b --output json`
- `cargo run -q -p inxi-cli -- -N -P -z --output json`

Tutte le verifiche sono passate.

E' stato inoltre avviato un primo set di fixture test sanitizzati per:

- `ip -brief address`
- `lsblk --json --bytes`
- `lspci -mm`
- `xrandr --query`
- `/proc/partitions`
- `/proc/self/mounts`
- `/proc/net/if_inet6`

## Safety guardrails

Sono stati aggiunti vincoli espliciti:

- sola lettura
- letture consentite solo sotto root sicure
- nessuna shell
- nessuna scrittura
- comandi esterni solo se auditati e risolti sotto path trusted

Documento relativo:

- `ai/inxi-safety-policy.md`

## Prossimo passo corretto

Entrare nel consolidamento post-M2:

1. stabilizzare il modello Rust
2. migliorare fallback e warning delle sezioni nuove
3. ampliare i fixture test reali
4. valutare il primo taglio di parallelismo interno
5. decidere il perimetro del futuro frontend TUI
