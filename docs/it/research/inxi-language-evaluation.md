# Valutazione Linguaggi per il Clone di `inxi`

## Scopo

Questo documento confronta una shortlist realistica di linguaggi contro i vincoli già fissati per la v1:

- Linux-first
- CLI
- output `screen` + `json`
- collector modulari
- parsing di `/proc`, `/sys`, `/dev`, `/etc`
- invocazione di tool esterni
- possibile estensione futura a BSD

Non valuta i linguaggi "in astratto".
Li valuta rispetto a questo progetto specifico.

## Shortlist considerata

- Rust
- Go
- Python
- Zig

Ho escluso volutamente linguaggi che non migliorano davvero il tradeoff rispetto a questi quattro.

## Criteri di valutazione

### 1. Modellazione del dominio

Quanto bene il linguaggio aiuta a rappresentare:

- sezioni
- warning
- stati del dato
- risultati parziali
- JSON stabile

### 2. Controllo del sistema operativo

Quanto è comodo lavorare con:

- file tree
- symlink
- metadata Unix/Linux
- processi figli
- env/cwd/stdin/stdout/stderr

### 3. Packaging e distribuzione

Quanto è facile distribuire il tool come:

- binario singolo
- installazione semplice
- dipendenze runtime minime

### 4. Velocità di sviluppo

Quanto è veloce arrivare a una v1 buona.

### 5. Manutenibilità a lungo termine

Quanto è probabile che il codice resti:

- leggibile
- testabile
- robusto
- facile da estendere con nuovi collector

### 6. Maturità e stabilità dell’ecosistema

Quanto sono affidabili:

- toolchain
- librerie core
- compatibilità nel tempo

## Osservazioni da fonti ufficiali

### Rust

Punti verificati:

- `std::process::Command` offre controllo completo su `cwd`, env, `stdin/stdout/stderr`, `spawn`, `output`, `status`
- `std::fs` copre il filesystem in modo portabile
- `std::os::unix::fs::MetadataExt` e `std::os::linux::fs::MetadataExt` espongono campi Unix/Linux-specific
- Serde fornisce derive per `Serialize`/`Deserialize` su struct ed enum
- la target policy di Rust distingue chiaramente i livelli di supporto delle piattaforme

Implicazione per il progetto:

- eccellente per modello dati forte
- eccellente per collector robusti
- molto buono per JSON stabile
- molto buono per supporto futuro a backend BSD/Linux distinti

### Go

Punti verificati:

- `os/exec` esegue comandi senza passare dalla shell per default
- `path/filepath.WalkDir` offre tree walking standard, deterministico e portabile
- `encoding/json` serializza struct esportate in oggetti JSON con comportamento standard
- il progetto Go mantiene una compatibility promise forte a livello source per Go 1

Implicazione per il progetto:

- ottimo per tool CLI di sistema
- ottimo per distribuire un binario singolo
- più rapido di Rust da portare in produzione
- meno espressivo di Rust nel modellare stati e varianti complesse

### Python

Punti verificati:

- `subprocess.run()` è l’API raccomandata per i processi figli
- `os.walk()` e `os.scandir()` sono solidi; `os.walk()` usa `os.scandir()` dalla 3.5 per ridurre chiamate `stat()`
- il modulo `json` standard è completo e configurabile
- Python ha una policy di backward compatibility esplicita per le API pubbliche

Implicazione per il progetto:

- eccellente per prototipo/reference implementation
- ottimo per parser e test fixtures
- meno adatto come clone finale se vuoi deployment semplice e zero runtime esterno
- più fragile sul piano delle performance e della distribuzione come tool “di sistema”

### Zig

Punti verificati:

- il linguaggio ha forti ambizioni sistemistiche e supporto target ampio
- però nelle note di rilascio ufficiali 0.13.0 il progetto dice esplicitamente che Zig è immaturo e che su progetti non banali è probabile dover partecipare al processo di sviluppo del linguaggio/toolchain

Implicazione per il progetto:

- interessante tecnicamente
- non adatto come scelta primaria per una v1 che deve produrre un clone affidabile
- rischio toolchain ed ecosistema troppo alto rispetto al valore aggiunto

## Matrice comparativa

Scala:

- 5 = eccellente
- 4 = molto buono
- 3 = buono con compromessi
- 2 = debole
- 1 = sconsigliato

| Criterio | Rust | Go | Python | Zig |
|---|---:|---:|---:|---:|
| Modellazione dominio | 5 | 4 | 3 | 3 |
| Controllo OS/processi/file | 5 | 5 | 4 | 4 |
| JSON e serializzazione | 5 | 4 | 4 | 2 |
| Packaging/distribuzione | 4 | 5 | 2 | 5 |
| Velocità di sviluppo v1 | 3 | 5 | 5 | 2 |
| Manutenibilità a lungo termine | 5 | 4 | 3 | 3 |
| Maturità/stabilità ecosistema | 5 | 5 | 5 | 2 |
| Supporto futuro Linux/BSD | 4 | 4 | 4 | 3 |
| Totale | 36 | 36 | 30 | 24 |

## Come leggere il pareggio Rust/Go

Il punteggio numerico uguale non significa equivalenza reale.
Significa che i due linguaggi vincono per motivi diversi.

### Rust vince se il criterio dominante è

- qualità architetturale
- modello dati rigoroso
- error handling forte
- evoluzione verso clone “serio” e duraturo
- supporto futuro a più backend senza degradare il design

### Go vince se il criterio dominante è

- arrivare rapidamente a una v1 utile
- minimizzare il costo cognitivo
- avere un binario singolo facilmente distribuibile
- mantenere una codebase semplice e leggibile per un tool operativo

## Analisi per questo progetto specifico

### Perché Rust è fortissimo qui

Il progetto non è solo “leggere file e stampare testo”.
Ha una struttura che beneficia moltissimo di:

- enum per gli stati del dato
- struct fortemente tipizzate per ogni sezione
- separazione forte tra raw source, normalized model e renderer
- serializzazione JSON naturale con Serde

Per un clone di `inxi`, questa parte conta molto.
Il dominio è pieno di:

- fallback
- warning
- dati parziali
- backend multipli
- campi opzionali
- rappresentazioni alternative dello stesso dato

Rust gestisce tutto questo molto bene.

### Perché Go è fortissimo qui

Detto questo, la v1 del clone è soprattutto:

- IO-bound
- process-bound
- parsing testuale
- orchestration-heavy

Non è un progetto CPU-bound.
Quindi il vantaggio di Rust sulle performance raw pesa poco nella fase iniziale.

Go invece porta vantaggi pragmatici molto forti:

- stdlib eccellente per processi, filesystem, JSON
- toolchain semplice
- curva di sviluppo più rapida
- binario singolo
- codice operativo molto lineare

Per una v1 Linux-first, Go è una scelta estremamente razionale.

### Perché Python non è la scelta finale giusta

Python è ottimo per:

- reverse engineering attivo
- fixture generation
- parser prototipali
- validazione del JSON target

Ma come implementazione finale del clone ha tre problemi strutturali:

1. distribuzione meno pulita
2. dipendenza dal runtime
3. minore disciplina architetturale se il progetto cresce

In pratica:

- ottimo linguaggio di supporto
- non il migliore come target finale del clone

### Perché Zig non è la scelta giusta adesso

Zig sarebbe interessante se l’obiettivo fosse:

- sperimentazione
- controllo low-level massimo
- toolchain-centric systems work

Ma qui l’obiettivo è costruire un clone affidabile di un tool maturo.
Il rischio introdotto da una toolchain ancora esplicitamente immatura non è giustificato.

## Raccomandazione finale

## Scelta migliore in assoluto

### Rust

Rust è la scelta migliore se vuoi costruire il clone “giusto”, non solo il clone “più veloce da scrivere”.

Motivo:

- è il miglior allineamento tra
  - modello dati
  - solidità dei collector
  - gestione errori
  - evoluzione futura
  - qualità architetturale

In altre parole:

- miglior scelta tecnica di lungo periodo

## Scelta migliore se vuoi arrivare prima a una v1 usabile

### Go

Se la priorità è:

- shipping rapido
- semplicità operativa
- binario singolo
- bassa complessità di implementazione

allora Go è probabilmente la scelta più pragmatica.

In altre parole:

- miglior scelta time-to-market

## Scelta da usare come supporto, non come target finale

### Python

Usalo se vuoi:

- scrivere parser di confronto
- generare fixture
- validare output
- costruire una reference implementation veloce

Non lo sceglierei come implementazione finale primaria del clone.

## Scelta da non raccomandare per questa v1

### Zig

Non per questo progetto, non adesso.

## Decisione pratica che consiglio

Se devo scegliere un solo linguaggio oggi, con l’obiettivo che hai descritto, la mia raccomandazione è:

### 1. Rust

Se vuoi fare il progetto bene e farlo durare.

### 2. Go

Se vuoi massimizzare la probabilità di avere presto una v1 valida con meno attrito.

## Decisione ancora più concreta

Se il progetto è:

- personale
- a risorse limitate
- con obiettivo di arrivare presto a un clone funzionante

allora Go è la scelta più economica.

Se il progetto è:

- strutturato
- orientato a qualità e lunga vita
- con disponibilità ad assorbire una maggiore complessità iniziale

allora Rust è la scelta migliore.

## La mia scelta, dovendo decidere ora

Io sceglierei:

### Rust

Con questa nota operativa:

- se vuoi ridurre il rischio di delivery, fai prima una spike o un prototipo di 2-3 sezioni in Rust
- se la velocità di avanzamento non è soddisfacente, Go resta il fallback più sensato

## Strategia alternativa molto solida

Una strategia molto pratica sarebbe:

1. JSON schema definitivo
2. prototipo mini in Python o Go per validare il flusso
3. implementazione definitiva in Rust

Ma se vuoi evitare doppio lavoro:

1. Rust diretto, se accetti un avvio più lento
2. Go diretto, se vuoi una v1 presto

## Fonti ufficiali usate

- Rust `std::process::Command`: <https://doc.rust-lang.org/std/process/struct.Command.html>
- Rust `std::fs`: <https://doc.rust-lang.org/stable/std/fs/>
- Rust `std::os::unix::fs::MetadataExt`: <https://doc.rust-lang.org/stable/std/os/unix/fs/trait.MetadataExt.html>
- Rust target tier policy: <https://doc.rust-lang.org/beta/rustc/target-tier-policy.html>
- Serde derive: <https://serde.rs/derive.html>
- `serde_json`: <https://docs.rs/serde_json/latest/serde_json/>
- Go `os/exec`: <https://pkg.go.dev/os/exec>
- Go `path/filepath`: <https://pkg.go.dev/path/filepath>
- Go `encoding/json`: <https://pkg.go.dev/encoding/json>
- Go compatibility promise: <https://go.dev/doc/go1compat>
- Python `subprocess`: <https://docs.python.org/3.10/library/subprocess.html>
- Python `os`: <https://docs.python.org/3.12/library/os.html>
- Python `json`: <https://docs.python.org/3/library/json.html>
- Python backward compatibility policy: <https://peps.python.org/pep-0387/>
- Zig 0.13.0 release notes: <https://ziglang.org/download/0.13.0/release-notes.html>
- Zig language documentation: <https://ziglang.org/documentation/master/>

## Conclusione in una riga

Per questo clone:

- `Rust` è la scelta migliore
- `Go` è la migliore alternativa pragmatica
