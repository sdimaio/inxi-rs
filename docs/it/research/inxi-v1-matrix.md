# Matrice V1 del Clone di `inxi`

## Obiettivo

Questa matrice congela il perimetro tecnico della v1.

Serve a rispondere a cinque domande:

1. quali sezioni entrano davvero in v1
2. quali fonti dati useremo per ciascuna sezione
3. quali dipendenze esterne accettiamo in v1
4. quali parti rimandiamo senza compromettere il valore del tool
5. quale ordine di implementazione conviene seguire

## Decisioni di perimetro per la v1

### Target primario

- Linux
- desktop e server comuni
- esecuzione da utente normale
- arricchimento opzionale se eseguito come root

### Output obbligatori

- `screen`
- `json`

### Output esclusi dalla v1

- `xml`
- `yaml`
- modalità IRC
- updater integrato
- debugger dataset upload

### Livelli di dettaglio della v1

- `basic`
- `normal`
- `extended`
- `admin`

Per la v1 non serve replicare perfettamente tutta la combinatoria di `-x`, `-xx`, `-xxx`.
Basta una mappatura coerente.

## Regole architetturali per la v1

### Regola 1

Prima i collector basati su file di sistema stabili.

### Regola 2

Poi i collector che dipendono da tool esterni molto diffusi.

### Regola 3

Ultimi i domini che richiedono fusione di fonti eterogenee o euristiche forti.

### Regola 4

Ogni sezione deve poter degradare in modo esplicito:

- `ok`
- `partial`
- `missing_tool`
- `permission_required`
- `unsupported`

## Classi di fonti dati

### Classe A: stabili e preferite

- `/proc/cpuinfo`
- `/proc/meminfo`
- `/proc/swaps`
- `/proc/uptime`
- `/proc/partitions`
- `/etc/os-release`
- `/sys/class/dmi/id/*`
- `/sys/devices/system/cpu/*`
- `/sys/class/net/*`
- `/sys/block/*`
- `/sys/class/drm/*`

### Classe B: buone ma variabili

- `df`
- `lsblk`
- `udevadm`
- `ip`
- `lspci`
- `xrandr`
- `xdpyinfo`

### Classe C: utili ma secondarie in v1

- `dmidecode`
- `smartctl`
- `glxinfo`
- `eglinfo`
- `vulkaninfo`

## Matrice sintetica delle sezioni

| Sezione | V1 | Valore | Fonti primarie | Fallback v1 | Tool esterni | Root utile | Difficoltà |
|---|---|---:|---|---|---|---|---|
| System | Core | alta | `uname`, `/etc/os-release`, `/proc/sys/kernel/hostname` | env desktop/sessione | nessuno | no | bassa |
| Machine | Core | alta | `/sys/class/dmi/id/*` | `dmidecode` admin | `dmidecode` | sì | bassa |
| CPU | Core | alta | `/proc/cpuinfo`, `/sys/devices/system/cpu/*` | `dmidecode` admin | nessuno nella base | sì | media |
| Memory | Core | alta | `/proc/meminfo` | `/sys/devices/system/memory/*`, `dmidecode` admin | `dmidecode` per moduli | sì | media |
| Graphics | Core | alta | `lspci`, `/sys/class/drm`, env display | `xrandr`, `xdpyinfo`, `glxinfo` | sì | a volte | alta |
| Network | Core | alta | `/sys/class/net/*` | `ip`, `lspci` | `ip`, `lspci` | no | media |
| Drives | Core | alta | `/proc/partitions`, `/sys/block/*`, `/dev/disk/by-id` | `udevadm`, `smartctl` admin | `udevadm`, `smartctl` | sì | alta |
| Partitions | Core | alta | `df`, `/proc/swaps`, `/dev/disk/by-*` | `lsblk` | `df`, `lsblk` | no | alta |
| Swap | Core | media | `/proc/swaps` | `swapon --show` | `swapon` fallback | no | bassa |
| Info | Core | media | aggregazione interna | shell/tty opzionale | nessuno | no | bassa |
| Audio | Stretch | media | `lspci`, `/proc/asound/*` | `lsusb`, processi server audio | `lspci`, `lsusb` | no | media |
| Battery | Stretch | media | `/sys/class/power_supply/*` | `upower` | `upower` fallback | no | bassa |
| USB | Stretch | media | `/sys/bus/usb/devices/*` | `lsusb` | `lsusb` | no | alta |
| Sensors | Stretch | media | `/sys/class/hwmon/*` | `sensors` | `sensors` | a volte | media |
| Bluetooth | Post-v1 | bassa | `lsusb`, sysfs | `bluetoothctl` | sì | no | media |
| RAID | Post-v1 | media | `/proc/mdstat` | `mdadm`, btrfs/zfs tool | sì | sì | alta |
| Logical | Post-v1 | media | `/dev/mapper`, sysfs | `lvs`, `lsblk` | sì | sì | alta |
| Repos | Post-v1 | bassa | distro-specific | package manager specifici | molti | no | alta |
| Processes | Post-v1 | bassa | `ps` | nessuno | `ps` | no | bassa |
| Weather | No | bassa | rete | API esterna | sì | no | non prioritaria |

## Matrice operativa dettagliata

### 1. `System`

#### Dati minimi v1

- hostname
- kernel release
- arch
- bits
- distro name
- desktop/session type se disponibile

#### Fonti v1

Ordine:

1. `uname`
2. `/etc/os-release`
3. `/proc/sys/kernel/hostname`
4. env:
   - `XDG_CURRENT_DESKTOP`
   - `DESKTOP_SESSION`
   - `WAYLAND_DISPLAY`
   - `DISPLAY`

#### Fallback

- se manca `os-release`, mostra solo kernel + hostname + platform

#### Note

- niente euristiche distro complicate in v1
- niente spin Ubuntu avanzati in v1

#### Difficoltà

- bassa

### 2. `Machine`

#### Dati minimi v1

- type
- vendor
- product
- version
- motherboard vendor/model/version
- firmware vendor/version/date
- seriale filtrato o marcato come privilegiato

#### Fonti v1

1. `/sys/class/dmi/id/*`
2. `dmidecode` solo per campi mancanti e solo in modalità admin/root

#### File principali

- `/sys/class/dmi/id/product_name`
- `/sys/class/dmi/id/product_version`
- `/sys/class/dmi/id/sys_vendor`
- `/sys/class/dmi/id/board_name`
- `/sys/class/dmi/id/board_vendor`
- `/sys/class/dmi/id/board_version`
- `/sys/class/dmi/id/bios_vendor`
- `/sys/class/dmi/id/bios_version`
- `/sys/class/dmi/id/bios_date`

#### Fallback

- se DMI non disponibile: sezione parziale, non errore fatale

#### Difficoltà

- bassa

### 3. `CPU`

#### Dati minimi v1

- model name
- vendor
- core count
- thread count
- arch
- min/max frequency se disponibili
- current/avg frequency se disponibile

#### Fonti v1

1. `/proc/cpuinfo`
2. `/sys/devices/system/cpu/*`

#### Fonti admin/stretch

- `dmidecode` per socket, family, stepping, microcode, cache raffinata

#### Fallback

- anche con solo `/proc/cpuinfo` la sezione deve essere utile

#### Difficoltà

- media

#### Rischi

- sistemi con topology incompleta
- laptop con freq dinamiche momentanee
- naming eterogeneo ARM/x86

### 4. `Memory`

#### Dati minimi v1

- total
- available
- used
- used percent

#### Fonti v1

1. `/proc/meminfo`

#### Fonti extend/admin

2. `/sys/devices/system/memory/*` per dettaglio extra
3. `dmidecode` per moduli RAM

#### Fallback

- se i moduli RAM non sono disponibili, non bloccare il report base

#### Difficoltà

- media

### 5. `Graphics`

#### Dati minimi v1

- lista GPU principali
- driver kernel noto
- protocollo display: `x11`, `wayland`, `tty`, `headless`
- server display se noto
- risoluzione attiva principale se nota

#### Fonti v1

1. `lspci`
2. `/sys/class/drm/*`
3. env:
   - `DISPLAY`
   - `WAYLAND_DISPLAY`
   - `XDG_SESSION_TYPE`
4. `xrandr` se X11 attivo

#### Fonti stretch

- `xdpyinfo`
- `glxinfo`
- `eglinfo`
- `vulkaninfo`

#### Cosa escludere dalla v1

- correlazione completa X driver / DRI / compositor / GL / EGL / Vulkan
- EDID completa
- mapping monitor-port molto raffinato

#### Strategia v1

Scomporre in 3 sottocollector:

- `GpuInventoryCollector`
- `DisplaySessionCollector`
- `MonitorResolutionCollector`

#### Difficoltà

- alta

### 6. `Network`

#### Dati minimi v1

- elenco interfacce significative
- tipo: ethernet/wifi/loopback/virtual
- stato: up/down
- MAC filtrato o parzialmente filtrato
- driver se noto
- IP locale opzionale e filtrato

#### Fonti v1

1. `/sys/class/net/*`
2. `ip -details addr`
3. `lspci` per associare device PCI principali

#### Fallback

- `ifconfig` se `ip` manca

#### Note

- niente WAN IP in v1 iniziale
- niente detection servizi di rete

#### Difficoltà

- media

### 7. `Drives`

#### Dati minimi v1

- inventario dischi fisici
- size
- model
- vendor se derivabile
- bus/type se noto (`SATA`, `NVMe`, `USB`)

#### Fonti v1

1. `/proc/partitions`
2. `/sys/block/*`
3. `/dev/disk/by-id/*`

#### Fonti extend/admin

4. `udevadm info`
5. `smartctl -i/-H/-A`

#### Strategia v1

Separare nettamente:

- inventario dischi fisici
- aggregazione totale storage

#### Cosa rinviare

- parsing SMART avanzato come fa `inxi`
- classificazione molto fine vendor/model
- temperature HDD/NVMe se richiedono molta logica

#### Difficoltà

- alta

### 8. `Partitions`

#### Dati minimi v1

- mount point
- device
- fs
- total
- used
- avail
- percent used
- label/uuid opzionali

#### Fonti v1

1. `df -P -T -k`
2. `/dev/disk/by-label/*`
3. `/dev/disk/by-uuid/*`

#### Fallback v1

4. `lsblk --json` o `lsblk -P` per completare label/uuid e hidden mounts

#### Scelte pragmatiche v1

- escludere overlay/tmpfs/devtmpfs e fs remoti dal totale storage
- supportare swap separatamente
- niente gestione avanzata fuse/stackable remota al livello di `inxi`

#### Difficoltà

- alta

### 9. `Swap`

#### Dati minimi v1

- device/file
- type
- total
- used
- priority se disponibile

#### Fonti v1

1. `/proc/swaps`

#### Fallback

2. `swapon --show --bytes` se necessario

#### Difficoltà

- bassa

### 10. `Info`

#### Dati minimi v1

- process count
- uptime
- shell o parent launcher se semplice da ottenere
- versione del clone

#### Fonti v1

1. `/proc`
2. `ps` solo se serve davvero
3. `/proc/uptime`

#### Scelte pragmatiche

- niente supporto IRC
- niente detection client complessa stile `StartClient`
- niente euristiche Konversation/shell wrapper

#### Difficoltà

- bassa

## Sezioni Stretch Goal

### `Audio`

Entrare solo se `lspci` + `/proc/asound` bastano a generare una sezione semplice.

### `Battery`

Molto fattibile su Linux via `/sys/class/power_supply/*`.
Può entrare in v1.1 senza impatto architetturale.

### `USB`

Valore alto ma costo alto.
Va fatta solo se modellata bene.

### `Sensors`

Va bene in v1.1 o v1.2.
La base Linux via `/sys/class/hwmon` è abbastanza pulita.

## Dipendenze esterne accettate in v1

### Obbligatorie di fatto

- `df`
- `uname`

### Fortemente raccomandate

- `ip`
- `lspci`
- `lsblk`

### Opzionali

- `udevadm`
- `dmidecode`
- `smartctl`
- `xrandr`
- `glxinfo`

## Politica di degradazione

Per ogni dipendenza esterna:

- se manca, la sezione deve restare utilizzabile
- il warning deve essere esplicito
- il JSON deve dire che il dato e' parziale

Esempi:

- `Graphics` senza `xrandr`: niente risoluzione monitor, ma GPU e sessione sì
- `Drives` senza `udevadm`: niente dettagli avanzati, ma inventario base sì
- `Machine` senza root: seriali schermati o non disponibili

## Privacy filters v1

### Da implementare subito

- IP locali
- MAC address
- seriali
- UUID filesystem
- label se richiesto
- path `/home/<user>`

### Da rinviare

- filtri vulnerabilità CPU
- filtri molto specifici per mount e kernel args

## Ordine di implementazione raccomandato

### Fase A: fondazione

1. modello dati
2. warning model
3. capability scanner
4. planner
5. JSON renderer
6. text renderer base

### Fase B: sezioni facili e ad alto valore

1. `System`
2. `Machine`
3. `CPU`
4. `Memory`
5. `Info`

### Fase C: sezioni core ma più complesse

1. `Network`
2. `Swap`
3. `Partitions`
4. `Drives`

### Fase D: dominio più rischioso

1. `Graphics`

### Fase E: stretch

1. `Battery`
2. `Audio`
3. `Sensors`
4. `USB`

## Ordine di implementazione alternativo se vogliamo una demo rapida

Se l'obiettivo e' mostrare presto qualcosa di utile:

1. `System`
2. `CPU`
3. `Memory`
4. `Network`
5. `Info`
6. `Machine`
7. `Partitions`
8. `Drives`
9. `Graphics`

Questo ordine produce una demo prima, ma tecnicamente e' meno pulito del precedente.

## Dataset di test minimi richiesti per la v1

### Fixture locali

- `/etc/os-release`
- `/proc/cpuinfo`
- `/proc/meminfo`
- `/proc/swaps`
- `/proc/partitions`
- `/sys/class/dmi/id/*`
- `/sys/class/net/*`
- `/sys/block/*`

### Fixture di comandi

- `df -P -T -k`
- `lsblk`
- `ip addr`
- `lspci`
- `xrandr`

### Fixture scenario

- laptop con batteria
- desktop con X11
- laptop con Wayland
- server headless
- sistema con NVMe
- sistema con disco USB

## Cosa deve essere chiuso prima della scelta del linguaggio

### Decisioni già chiudibili

- target v1: Linux
- output v1: screen + JSON
- sezioni core: `System`, `Machine`, `CPU`, `Memory`, `Graphics`, `Network`, `Drives`, `Partitions`, `Swap`, `Info`
- niente XML/IRC/updater/debugger in v1

### Decisioni ancora da chiudere

- JSON schema esatto
- contratto dei warning
- sintassi CLI finale del clone
- livello di compatibilità nominale con `inxi`

## Decisione raccomandata

La raccomandazione concreta e':

- v1 Linux-only
- 10 sezioni core
- zero dipendenze obbligatorie oltre ai comandi base di sistema
- dipendenze esterne solo come arricchimento
- JSON come API stabile del progetto

## Prossimo passo corretto

Ora che il perimetro v1 è fissato, il passo più utile è uno di questi:

1. definire lo schema JSON definitivo
2. definire le interfacce dei collector
3. fare la lista comparativa dei linguaggi contro questa matrice

Il punto giusto per parlare di linguaggi è adesso, non prima.
