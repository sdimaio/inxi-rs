# Specifica Operativa del Clone di `inxi`

## Scopo

Questo documento prende il reverse engineering e lo converte in una specifica implementabile.

Assume `Rust` come linguaggio di implementazione.
Definisce:

- cosa deve fare il clone
- cosa puo' essere rimandato
- quale modello dati interno serve
- quale compatibilita' CLI ha senso avere
- in che ordine conviene implementare
- quali vincoli architetturali servono per una futura TUI

## Obiettivo reale

Il clone non deve essere:

- una traduzione 1:1 del Perl
- un formatter di output simile a `inxi`

Il clone deve essere:

- un tool CLI multipiattaforma
- con collector modulari
- con output umano leggibile
- con output strutturato stabile
- con una superficie compatibile con `inxi` dove utile

## Livelli di compatibilita'

### Compatibilita' forte

Da preservare:

- semantica delle sezioni principali
- livelli di dettaglio
- fallback robusti
- gestione di tool mancanti
- filtri privacy
- output sintetico utile per supporto tecnico

### Compatibilita' debole

Puo' differire:

- identico wording delle etichette
- identica disposizione visuale del testo
- identiche chiavi JSON interne di `inxi`
- supporto a casi legacy estremi

### Compatibilita' da evitare

Non copiare:

- stato globale diffuso
- chiavi layout-encoded tipo `000#1#0#System`
- coupling tra renderer e modello dati
- inferenze di contesto che cambiano il comportamento del formato dati

## Perimetro della versione 1

La v1 deve coprire il caso d'uso Linux desktop/server comune.

### Sezioni obbligatorie

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

### Sezioni utili ma rinviabili

- `Audio`
- `Battery`
- `USB`
- `Sensors`
- `RAID`
- `Logical`
- `Repos`
- `Processes`
- `Bluetooth`
- `Weather`

### BSD

BSD va progettato dalla v1, ma non deve bloccare la prima release.

Scelta pratica:

- definire interfacce multipiattaforma subito
- implementare Linux prima
- aggiungere backend BSD dopo i collector Linux stabili

## Output da supportare in v1

### Obbligatori

- output umano su terminale
- JSON stabile

### Rinviabili

- XML
- YAML
- TUI/interactive output

## Vincolo architetturale per la futura TUI

La TUI non fa parte della v1, ma deve influenzare il design fin da subito.

Regola:

- il progetto non deve nascere come binario CLI monolitico
- deve nascere come core riusabile con frontend separati

Struttura consigliata:

- `core`/library: modello dati, capability scanner, planner, collector, filtri privacy
- `cli` frontend: parsing argomenti, esecuzione richiesta, rendering testuale e JSON
- `tui` frontend futuro: navigazione interattiva, refresh, pannelli e drill-down

Vincoli pratici:

- collector e planner non devono leggere o scrivere direttamente il terminale
- il renderer testuale non deve contenere logica di raccolta dati
- il modello dati deve essere sufficiente sia per output lineare sia per viste interattive
- errori, warning e source trace devono essere strutturati e non solo formattati come testo

Obiettivo:

- poter aggiungere in futuro una TUI in stile classico a caratteri senza riscrivere il motore

## Invarianti di sicurezza

Il clone deve restare strettamente osservativo.

Regole:

- nessuna scrittura su filesystem
- nessuna scrittura su `/proc`, `/sys`, `/dev`
- nessuna shell
- nessuna privilege escalation
- nessun accesso di rete
- eventuali comandi esterni solo se auditati, in whitelist e con argomenti fissi

Conseguenza architetturale:

- ogni nuova fonte dati deve passare da API di accesso sicure, non da chiamate libere a `std::fs` o `Command`

## Contratto di output JSON

Il JSON del clone deve rappresentare il dominio, non il layout.

Schema concettuale:

```json
{
  "meta": {
    "tool": "clone-name",
    "version": "x.y.z",
    "host": "hostname",
    "timestamp": "ISO-8601",
    "platform": "linux"
  },
  "sections": {
    "system": {},
    "machine": {},
    "cpu": {},
    "memory": {},
    "graphics": {},
    "network": {},
    "drives": {},
    "partitions": {},
    "swap": {},
    "info": {}
  },
  "warnings": [],
  "capabilities": {}
}
```

Regole:

- chiavi stabili
- niente encoding dell'ordine nel nome delle chiavi
- unità esplicite dove necessario
- array per oggetti ripetibili
- distinguere `null`, `unknown`, `unavailable`, `permission_required`

## Modello dati interno

### Tipi top-level

- `Report`
- `Meta`
- `Warning`
- `CapabilityReport`

### Sezioni

- `SystemSection`
- `MachineSection`
- `CpuSection`
- `MemorySection`
- `GraphicsSection`
- `NetworkSection`
- `DrivesSection`
- `PartitionsSection`
- `SwapSection`
- `InfoSection`

### Oggetti di dominio

- `KernelInfo`
- `DistroInfo`
- `DesktopInfo`
- `FirmwareInfo`
- `MotherboardInfo`
- `CpuTopology`
- `CpuSpeedInfo`
- `GpuDevice`
- `DisplayServerInfo`
- `MonitorInfo`
- `GraphicsApiInfo`
- `NetworkInterface`
- `PhysicalDisk`
- `Partition`
- `SwapDevice`
- `MountInfo`
- `MemoryInfo`

### Tipi ausiliari

- `DataState`
  - `ok`
  - `missing`
  - `permission_required`
  - `unsupported`
  - `unknown`
- `SourceTrace`
  - da dove viene il dato

## Regola fondamentale del modello

Ogni collector deve restituire due cose:

1. dati normalizzati
2. metadati di provenienza e qualità

Questo evita di perdere informazione come:

- dato mancante
- dato non disponibile su piattaforma
- dato presente ma richiede root
- dato inferito e non letto direttamente

## CLI del clone

### Strategia

Non serve partire con tutta la CLI di `inxi`.
Serve una CLI compatibile per il core.

### Opzioni v1

- `-b` / `--basic`
- `-S` / `--system`
- `-M` / `--machine`
- `-C` / `--cpu`
- `-m` / `--memory`
- `-G` / `--graphics`
- `-N` / `--network`
- `-D` / `--disk`
- `-P` / `--partition`
- `-j` / `--swap`
- `-I` / `--info`
- `-v N`
- `-x`, `-xx`, `-xxx`
- `-a` / `--admin`
- `-z` / `--filter`
- `-Z` / `--no-filter`
- `--output json|screen`

### Opzioni da rinviare

- updater
- debugger dataset
- weather
- export XML
- supporto IRC
- opzioni molto specifiche di forcing backend

## Livelli di dettaglio

Il clone deve mantenere un concetto di dettaglio simile a `inxi`, ma con semantica piu' pulita.

Proposta:

- `detail=basic`
- `detail=normal`
- `detail=extended`
- `detail=full`
- `detail=admin`

Mapping CLI:

- `-b` => `basic`
- default sezioni selezionate => `normal`
- `-x` => `extended`
- `-xx` => `full`
- `-a` => `admin`

## Planner

Il planner deve essere un componente esplicito.

Input:

- opzioni CLI
- contesto runtime
- capability scanner

Output:

- sezioni richieste
- livello dettaglio
- collector necessari
- dipendenze
- privilegi richiesti

Struttura concettuale:

- `Request`
- `ExecutionPlan`
- `CollectorTask`

## Capability scanner

Il clone deve eseguire un solo discovery iniziale di:

- OS
- disponibilita' tool
- privilegi effettivi
- accesso a file/path chiave
- sessione grafica o no

Output:

- `CapabilityMatrix`

Campi minimi:

- `platform`
- `is_root`
- `has_display`
- `display_protocol`
- `commands`
- `paths`
- `modules_optional`

## Collector architecture

Ogni collector deve implementare un contratto semplice:

- `supports(request, capabilities) -> bool`
- `collect(context) -> result`

Ogni collector:

- non stampa
- non tocca argomenti CLI globali
- non dipende da stato mutabile implicito
- dichiara fonti e fallback

## Backend per dominio

### System

Fonti Linux:

- `uname`
- `/etc/os-release`
- `/proc/sys/kernel/hostname`
- env/display/session

### Machine

Fonti Linux:

- `/sys/class/dmi/id/*`
- `dmidecode` come fallback/admin

### CPU

Fonti Linux:

- `/proc/cpuinfo`
- `/sys/devices/system/cpu/*`
- `dmidecode` per dettagli aggiuntivi

### Memory

Fonti Linux:

- `/proc/meminfo`
- `/sys/devices/system/memory/*`
- `dmidecode` per moduli

### Graphics

Fonti Linux:

- `lspci`
- `/sys/class/drm`
- `xrandr`
- `xdpyinfo`
- `xprop`
- `glxinfo`
- `eglinfo`
- `vulkaninfo`

### Network

Fonti Linux:

- `ip`
- `ifconfig` fallback
- `/sys/class/net/*`
- `lspci`
- `lsusb`

### Drives

Fonti Linux:

- `/proc/partitions`
- `/sys/block/*`
- `/dev/disk/by-id`
- `udevadm`
- `smartctl`

### Partitions / Swap

Fonti Linux:

- `df`
- `/proc/swaps`
- `lsblk`
- `/dev/disk/by-label`
- `/dev/disk/by-uuid`

## Normalizzazione

I parser di comandi esterni devono essere isolati in adapter specifici.

Esempio:

- `LspciAdapter`
- `LsusbAdapter`
- `SmartctlAdapter`
- `XrandrAdapter`
- `GlxinfoAdapter`

Mai mischiare:

- esecuzione comando
- parsing testo
- mapping su modello dominio

## Filtri privacy

Da supportare fin dalla v1:

- IP locali
- IP WAN
- seriali
- UUID
- label sensibili
- username in mount path tipo `/home/<user>`

La privacy non deve essere un post-processing testuale fragile.
Deve avvenire sul modello dati prima del rendering.

## Error model

Ogni sezione deve poter restituire:

- dati completi
- dati parziali
- errore recuperabile
- non supportato

Ogni warning deve avere:

- `code`
- `message`
- `section`
- `source`

Esempi:

- `missing_tool`
- `permission_required`
- `unsupported_platform`
- `parse_error`
- `incomplete_data`

## Rendering umano

Il renderer testuale deve essere separato dal modello.

Linee guida:

- blocchi per sezione
- chiavi stabili
- wrapping terminale
- colori opzionali
- nessuna logica di raccolta durante il render

## Test strategy

### Unit test

- parser di file `/proc`
- parser di file `/sys`
- parser di output comandi
- filtri privacy
- planner

### Fixture test

- dataset reali Linux
- dataset sintetici edge case
- fixture con tool mancanti
- fixture con permessi insufficienti

### Golden test

- output JSON
- output testuale per subset stabili

## Milestone proposte

### M1

- modello dati
- capability scanner
- planner
- JSON renderer
- sezioni: `System`, `Machine`, `CPU`, `Memory`, `Info`

### M2

- `Graphics`, `Network`, `Drives`, `Partitions`, `Swap`
- renderer testuale usabile
- privacy filters

### M3

- `Audio`, `Battery`, `USB`, `Sensors`
- piu' fallback
- admin detail

### M4

- BSD backend iniziale
- compatibilita' CLI piu' ampia
- ottimizzazione e parallelismo mirato

## Criteri di successo

La v1 e' riuscita se:

- su Linux comune produce un report utile quanto `inxi -b` e `inxi -F` sulle sezioni coperte
- il JSON e' stabile e pulito
- la codebase e' modulare
- aggiungere una nuova fonte o un nuovo backend non richiede toccare il renderer

## Direzione confermata

Decisioni ormai fissate:

1. linguaggio: `Rust`
2. v1: `Linux-first`, `screen` + `json`
3. architettura: core headless con frontend CLI, TUI rinviata ma prevista

## Prossimo passo operativo

Il passo corretto adesso e' entrare nel design Rust concreto:

1. layout crate/workspace
2. modello tipi Rust del report
3. capability scanner Linux
4. primi collector `System`, `Machine`, `CPU`, `Memory`, `Info`
5. renderer `json` e renderer testuale minimo
