# Reverse Engineering di `inxi`

## Obiettivo

Questo documento descrive `inxi` dal punto di vista architetturale e operativo con un obiettivo preciso:
capire come clonarlo in un altro linguaggio senza limitarsi a una traduzione 1:1 del file Perl.

Non contiene ancora una scelta del linguaggio di destinazione. Quella viene dopo.

## Sintesi esecutiva

`inxi` e' un sistema di raccolta informazioni hardware/software costruito come script Perl monolitico, ma organizzato internamente come insieme di pseudo-moduli `package`.

Caratteristiche strutturali principali:

- Un solo file eseguibile: `inxi`
- Circa 39.5k linee
- 51 `package`
- 625 `sub`
- Stato globale condiviso quasi ovunque
- Forte dipendenza da:
  - filesystem virtuali di sistema (`/proc`, `/sys`, `/dev`, `/etc`, `/var`)
  - tool esterni (`lspci`, `lsusb`, `dmidecode`, `smartctl`, `xrandr`, `glxinfo`, `ip`, `ifconfig`, `udevadm`, `sysctl`, ecc.)
  - euristiche e fallback multipiattaforma

Il valore reale di `inxi` non sta nel rendering testuale, ma in tre asset:

1. il grafo di fallback tra fonti dati
2. la normalizzazione di output molto eterogenei
3. il modello implicito di dati che viene poi reso in screen/json/xml

Il clone non dovrebbe copiare il monolite. Dovrebbe separare:

- parsing CLI
- piano di raccolta
- collector per dominio
- normalizzazione tipizzata
- renderer

## Fatti osservati nel repository

- Entry point: `main()` in `inxi`
- Versione hardcoded nello script:
  - `3.3.40`
  - data `2025-11-25`
- Repository minimale:
  - `inxi`
  - `README.txt`
  - `inxi.1`
  - changelog e metadata

Questo significa che la "vera" architettura e' tutta interna al singolo script.

## Flusso di esecuzione

Pipeline ad alto livello:

1. `main()`
2. `initialize()`
3. `StartClient::set()`
4. `OptionsHandler::get()`
5. `CheckTools::set()`
6. setup colori/separatori
7. `OutputGenerator::generate()`
8. cleanup

In pratica:

- `initialize()` costruisce il contesto di base:
  - path binari
  - path utente
  - file di sistema noti
  - OS/base platform
  - config
  - dimensioni terminale
- `OptionsHandler::get()` non raccoglie dati:
  - popola flag globali
  - poi `post_process()` traduce i flag utente in dipendenze reali
- `CheckTools::set()` verifica quali comandi di supporto esistono e con quali permessi
- `OutputGenerator::generate()` esegue solo i collector necessari in base ai flag attivi

Questo e' gia' un indizio importante per il clone:
`inxi` non e' "imperativo puro", ma un piccolo motore di pianificazione guidato da flag.

## Modello mentale corretto di `inxi`

Il modello corretto non e':

- "uno script che stampa roba"

Il modello corretto e':

- "un orchestratore di collector eterogenei con rendering finale"

Architettura implicita:

- CLI layer
- capability detection
- source selection / fallback
- collection
- normalization
- presentation

Nel codice queste fasi sono intrecciate, ma sono chiaramente presenti.

## Stato globale

Il design interno dipende fortemente da hash/array globali, tra cui:

- `%show`
  - cosa mostrare
- `%use`
  - quali sottosistemi attivare
- `%force`
  - override manuali
- `%alerts`
  - stato dei tool esterni: presente, mancante, permessi, path
- `%loaded`
  - memoization dei collector gia' eseguiti
- `%devices`, `%usb`, `%sysctl`, `%service_tool`, `%program_values`
  - cache/indici di dominio
- `@partitions`, `@swaps`, `@proc_partitions`, `@ps_aux`, `@ps_cmd`
  - dataset intermedi condivisi

Conseguenza:

- il comportamento e' efficiente per script monolitico
- ma difficile da testare in modo isolato
- e molto difficile da portare fedelmente senza reintrodurre accoppiamenti inutili

## Parser CLI e pianificazione

`OptionsHandler::get()` definisce un parser dichiarativo con `Getopt::Long`.

L'utente non attiva collector direttamente.
Attiva combinazioni di flag come:

- `-b`
- `-C`
- `-G`
- `-s`
- `-v N`
- `-x`, `-xx`, `-xxx`
- `-a`

Poi `post_process()`:

- valida i modifier
- abilita dipendenze derivate
- seleziona i tool da usare
- attiva fallback e permessi
- imposta gruppi di feature BSD/Linux

Esempi:

- se mostri disco o partizioni:
  - `use{block-tool}`
- se mostri RAM o machine con certi livelli:
  - `use{dmidecode}`
- se mostri audio/grafica/network:
  - `use{pci}`
- se mostri USB/audio/network/grafica/dischi:
  - `use{usb}`

Per il clone questo e' il primo pezzo da rifattorizzare come:

- `RequestedFeatures`
- `DerivedPlan`
- `CapabilityMatrix`

## Capability detection

`CheckTools::set()` costruisce una tabella di tool richiesti per il run corrente.

Per ogni tool determina:

- presente e usabile
- presente ma con errore permessi
- mancante
- non applicabile alla piattaforma

Questo stato viene poi riusato da tutto il resto del programma.

Pattern corretto da copiare nel clone:

- discovery iniziale
- memoization capability
- niente probing ripetuto in ogni collector

## Domini funzionali interni

I package principali possono essere raggruppati cosi'.

### Orchestrazione

- `CheckTools`
- `Configs`
- `OptionsHandler`
- `StartClient`
- `OutputGenerator`
- `SystemDebugger`

### Collector "top-level item"

Ogni item produce un blocco funzionale:

- `AudioItem`
- `BatteryItem`
- `BluetoothItem`
- `CpuItem`
- `DriveItem`
- `GraphicItem`
- `LogicalItem`
- `MachineItem`
- `NetworkItem`
- `OpticalItem`
- `PartitionItem`
- `ProcessItem`
- `RaidItem`
- `RamItem`
- `RepoItem`
- `SensorItem`
- `SlotItem`
- `SwapItem`
- `UnmountedItem`
- `UsbItem`
- `WeatherItem`

### Collector / helper di basso livello

- `DesktopData`
- `DeviceData`
- `DiskDataBSD`
- `DistroData`
- `DmData`
- `DmidecodeData`
- `GlabelData`
- `InitData`
- `IpData`
- `KernelCompiler`
- `KernelParameters`
- `LsblkData`
- `MemoryData`
- `PackageData`
- `PartitionData`
- `PowerData`
- `ProgramData`
- `PsData`
- `ServiceData`
- `ShellData`
- `UsbData`
- `ParseEDID`

L'idea giusta per il clone e':

- gli `*Item` diventano use-case o report section
- i `*Data` diventano collector/provider/adapter

## Pattern di raccolta dati

`inxi` usa quasi sempre questo pattern:

1. tentativo fonte primaria
2. fallback fonte secondaria
3. fallback tool esterno
4. messaggio di incompletezza o assenza

Esempi osservati:

- CPU:
  - `/proc/cpuinfo`
  - `/sys/devices/system/cpu/...`
  - `dmidecode` per alcuni dettagli
  - `sysctl` su BSD
- Dischi:
  - `/proc/partitions`
  - `/sys/block/...`
  - `/dev/disk/by-id`
  - `smartctl`
  - `udevadm`
  - `fdisk`
  - `gpart`/`disklabel` su BSD
- Grafica:
  - `lspci` / `pciconf` / `pcidump` / `pcictl`
  - `/sys/class/drm`
  - `xrandr`, `xdpyinfo`, `xdriinfo`, `xprop`
  - `glxinfo`, `eglinfo`, `vulkaninfo`
  - Wayland: `wayland-info`, `wlr-randr`, `swaymsg`
- USB:
  - `/sys/bus/usb/devices`
  - `lsusb`
  - `usbdevs`, `usbconfig`
- Distro:
  - `/etc/os-release`
  - `/etc/lsb-release`
  - `/etc/issue`
  - distro-specific release files

Questo pattern non va banalizzato.
E' la parte piu' costosa da ricostruire bene in un clone.

## Fonti dati: classi reali

Dal punto di vista del clone conviene classificare le fonti cosi':

### 1. File strutturati quasi stabili

- `/proc/cpuinfo`
- `/proc/meminfo`
- `/proc/swaps`
- `/etc/os-release`
- molti file in `/sys/class/...`

Questi sono i piu' adatti a parser tipizzati.

### 2. File semi-strutturati con varianti di kernel/distro

- `/proc/partitions`
- `/proc/asound/cards`
- `/var/log/Xorg.0.log`
- vari file DMI/sysfs

Questi richiedono parser resilienti.

### 3. Output di comandi esterni

- `lspci`
- `lsusb`
- `dmidecode`
- `smartctl`
- `xrandr`
- `glxinfo`
- `vulkaninfo`
- `ip`
- `ifconfig`
- `sysctl`

Questi sono i piu' fragili e i piu' dipendenti dalla piattaforma/versione.

### 4. Euristiche ambientali

- shell/terminal/tty
- desktop environment
- display manager
- protocollo grafico
- client IRC

Questa parte non e' solo data collection; e' inferenza.

## Il sistema grafico e' il dominio piu' complesso

`GraphicItem` e' uno dei package piu' densi.

Problemi gestiti:

- schede PCI e device USB video
- X11 vs Wayland vs console
- driver kernel vs driver X vs DRI vs API GL/EGL/Vulkan
- monitor, porte, risoluzioni, EDID
- compositori
- casi "unsafe" per alcune API/hardware

Per il clone va trattato come sottosistema separato, non come semplice collector.

Serve almeno questa scomposizione:

- GPU device discovery
- display server discovery
- monitor topology discovery
- graphics API discovery
- driver correlation

## Storage e partizioni sono il secondo dominio piu' complesso

`DriveItem`, `PartitionItem`, `RaidItem`, `LogicalItem` lavorano insieme.

Problemi gestiti:

- distinguere dischi fisici, partizioni, LVM, mapper, RAID, zram, swap file
- sommare capacita' logiche evitando doppio conteggio
- leggere label/uuid
- capire bus/periferica USB o SATA/NVMe
- rilevare SMART e normalizzarlo
- gestire eccezioni BSD/Linux

Osservazione importante:

`inxi` non ha un singolo "modello storage". Lo ricostruisce fondendo piu' viste parziali.

Per il clone serve un modello esplicito:

- `PhysicalDisk`
- `LogicalDisk`
- `Partition`
- `SwapDevice`
- `RaidArray`
- `FilesystemMount`

## Distro detection: fortemente euristica

`DistroData` non fa solo parsing di `/etc/os-release`.

Fa anche:

- merge tra `os-release`, `lsb-release`, `issue`, file custom
- riconoscimento distro derivate
- deduzione della base distro
- gestione spin Ubuntu
- tabelle di mapping Debian/Devuan/Ubuntu

Questa logica va isolata in un modulo completamente separato.
Non e' essenziale per una prima MVP del clone, ma e' essenziale per la compatibilita' percepita.

## USB e PCI: normalizzazione multi-backend

`DeviceData` e `UsbData` fanno una cosa molto utile:

- leggono fonti diverse
- convertono ogni record in una struttura indice-posizionale ricorrente
- poi classificano il device per dominio

Di fatto fanno da "bus inventory layer".

Nel clone va sostituito con strutture nominate, non posizionali.

Esempio di campi impliciti oggi:

- tipo device
- class ID
- bus ID
- device name
- vendor/product ID
- driver
- moduli alternativi
- seriale
- porte/velocita'

## Modello dati di output

L'output interno e' piu' strutturato di quanto sembri.

Le chiavi sono nel formato:

- `NNN#container#indent#label`

Dove:

- `NNN` impone l'ordine
- `container` indica il comportamento di rendering
- `indent` controlla la profondita'
- `label` e' la chiave semantica

Esempio osservato nell'export JSON:

- `000#1#0#System`
- `001#1#1#Kernel`
- `003#0#2#bits`

Questo conferma che:

- il JSON attuale non e' un vero schema pubblico
- e' la serializzazione del renderer interno
- non e' il formato giusto da copiare per il clone

Per il clone serve un modello tipizzato vero, ad esempio:

- `Report`
- `Section`
- `Entry`
- `Value`

E solo dopo:

- `ScreenRenderer`
- `JsonRenderer`
- `XmlRenderer`

## Rendering e presentazione

`print_data()` e `print_basic()` mescolano:

- layout
- wrapping
- colori
- numerazione
- prefissi `Device-1`, `Monitor-2`, `ID-1`
- gestione di terminale/IRC

La separazione tra dominio e rendering e' parziale.

Per il clone il renderer deve essere l'ultimo layer, non il punto in cui si scopre la semantica dei dati.

## Modalita' operative osservate

### Output screen

Con `./inxi -b` l'output e':

- leggibile
- compatto
- orientato a supporto tecnico umano

### Output JSON

Con PTY attivo, `./inxi -b --output json --output-file print` produce JSON valido.

Senza PTY, in questo ambiente, il tool puo' attivare logiche da "IRC client" e rifiutare l'export strutturato.

Questo e' un punto progettualmente importante:

- il rilevamento del contesto di esecuzione e' troppo accoppiato all'output mode

Nel clone:

- niente coupling tra contesto terminale e formato dati

### Output XML

L'XML dipende da `XML::Dumper`, non installato qui.
Quindi l'export XML non e' self-contained.

### Debug

`--debug 3` mostra:

- tracing funzione per funzione
- tempi cumulativi
- sequenza reale dei collector invocati

Questo e' molto utile per validare il call graph effettivo.

## Accoppiamenti problematici da non replicare

### 1. Stato globale ovunque

Problema:

- difficile testare
- difficile comporre
- difficile eseguire collector in parallelo

### 2. Strutture positional-array

Molti dataset usano array con semantica implicita per indice.

Problema:

- poco leggibile
- fragile in manutenzione
- porting piu' rischioso

### 3. Rendering-aware data model

Il modello interno porta informazione di ordine e indentazione.

Problema:

- JSON/XML non rappresentano il dominio ma il layout

### 4. Capability, planning e collection intrecciati

`show/use/force/alerts/loaded` funzionano bene, ma sono troppo distribuiti.

Problema:

- difficile sapere "perche' questo collector e' partito?"

### 5. Euristiche sparse

Molte euristiche sono giuste, ma sono inglobate nel collector locale invece che centralizzate.

Problema:

- difficile mantenere coerenza

## Cosa va assolutamente preservato

Nel clone bisogna preservare:

- fallback multipli per dominio
- copertura Linux/BSD dove possibile
- robustezza rispetto a tool mancanti
- concetto di output "incomplete but useful"
- filtri privacy
- livelli di dettaglio (`basic`, `full`, `extra`, `admin`)
- capacita' di export strutturato

## Cosa NON vale la pena clonare 1:1 in una prima iterazione

- supporto IRC nativo
- updater integrato del binario
- debugger/upload FTP
- XML basato su modulo opzionale
- enorme matrice storica di distro legacy e DE/WM esotici

Questi si possono reintrodurre dopo.

## Strategia raccomandata per il clone

### Fase 1: definire il modello dominio

Prima del codice di raccolta servono strutture chiare:

- `Report`
- `SystemSection`
- `MachineSection`
- `CpuSection`
- `MemorySection`
- `GraphicsSection`
- `StorageSection`
- `NetworkSection`
- `AudioSection`
- `BatterySection`
- `ProcessSection`
- `RepoSection`

Ogni section deve essere serializzabile senza informazione di layout.

### Fase 2: introdurre un planner esplicito

Input:

- opzioni utente
- contesto piattaforma
- capability dei tool

Output:

- lista collector da eseguire
- ordine
- prerequisiti

### Fase 3: implementare collector per dominio

Ogni collector dovrebbe:

- dichiarare fonti possibili
- usare timeout e gestione errori
- restituire tipi nominati
- non stampare mai

### Fase 4: normalizzazione

Separare sempre:

- raw source parsing
- normalized domain objects

### Fase 5: renderer

Render separati:

- human/terminal
- json
- eventualmente yaml/xml

## MVP realistica del clone

Una MVP ragionevole non deve copiare tutto `inxi`.

Ordine consigliato:

1. `System`
2. `Machine`
3. `CPU`
4. `Memory`
5. `Graphics` minima
6. `Network`
7. `Drives` minima
8. `Partitions` minima

Solo dopo:

- battery
- sensors
- repo/package counts
- RAID/logical
- graphics API avanzate
- distro base inference avanzata

## Matrice di difficolta' per dominio

### Bassa

- uptime
- hostname
- kernel
- memoria base
- distro base da `os-release`

### Media

- CPU
- network
- battery
- package counts

### Alta

- graphics
- storage completo
- USB completo
- RAID/logical
- distro detection completa
- shell/client/context detection

## Test strategy che il clone dovra' avere

`inxi` contiene una grande quantita' di logica difensiva che oggi vive nel codice stesso.
Nel clone va trasformata in test.

Servono almeno:

- fixture di file `/proc` e `/sys`
- fixture di output comandi esterni
- golden output per JSON
- test per capability matrix
- test per fallback selection
- test per privacy filtering

Idealmente:

- dataset reali anonimizzati per Linux e BSD

## Conclusione

La conclusione tecnica e' chiara:

- `inxi` non va "tradotto"
- va "smontato e ricostruito"

La parte da preservare e':

- conoscenza pratica delle fonti dati
- fallback
- euristiche utili
- ergonomia dell'output

La parte da rifare e':

- modello dati
- orchestrazione
- isolamento dei collector
- testabilita'
- separazione netta tra dati e rendering

## Backlog immediato per il progetto clone

1. Formalizzare il perimetro della prima versione compatibile
2. Disegnare schema dati del report
3. Disegnare capability matrix e planner
4. Mappare per ogni sezione le fonti da supportare in MVP
5. Solo dopo: scegliere il linguaggio
