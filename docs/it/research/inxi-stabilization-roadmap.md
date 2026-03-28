# inxi-rs Stabilization Roadmap

## Stato attuale

Data: `2026-03-27`

Gia' completato:

- workspace Rust separato da `inxi`
- core headless + CLI
- policy di safety `read-only`
- sezioni v1 operative:
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
- output `screen` + `json`
- filtro privacy `-z`
- fallback `procfs` per `Network`, `Drives`, `Partitions`
- `--self-check`
- fixture parser test
- golden test per renderer normale e `self-check`
- refactor collector in moduli:
  - `base`
  - `graphics`
  - `network`
  - `storage`

## Cosa manca prima di chiamarla stabile

### Bloccanti veri per `v0.1.0`

1. Comparazione sistematica con `inxi`
   - eseguire confronti sezione per sezione su almeno 2-3 macchine Linux diverse
   - classificare differenze in:
     - bug nostri
     - differenze accettabili
     - feature non ancora implementate

2. Harden dei warning e dei fallback
   - rendere piu' uniforme il catalogo codici warning
   - distinguere meglio `missing`, `partial`, `fallback`, `permission_required`
   - verificare che ogni fallback lasci traccia coerente in `screen`, `json`, `self-check`

3. Sweep di compatibilita' CLI
   - rivedere mapping opzioni rispetto alla v1 scelta
   - sistemare `--help`, `--version`, errori d'uso, combinazioni di flag
   - fissare il contratto minimo delle opzioni supportate in `v0.1.0`

4. Test matrix piu' ampia
   - aggiungere fixture reali sanitizzate per casi edge:
     - sistemi senza `xrandr`
     - sistemi senza `lspci`
     - sistemi senza `lsblk`
     - host senza GUI
     - host con storage NVMe + SATA
     - host con interfacce virtuali

5. Review del modello JSON
   - congelare i nomi campo della v1
   - evitare cambi di schema dopo il primo rilascio stabile
   - definire cosa e' garantito e cosa e' best effort

6. Documentazione minima di rilascio
   - README del clone Rust
   - safety model
   - sezioni supportate
   - limiti noti

### Fortemente consigliati prima di `v0.1.0`

7. Snapshot comparativi reali
   - raccogliere alcuni output anonimi reali in `tests/fixtures`
   - usarli come regression pack oltre ai golden sintetici

8. Piccola pulizia architetturale finale
   - estrarre eventualmente i renderer in moduli dedicati
   - ridurre ancora accoppiamenti interni se emergono durante il confronto con `inxi`

9. Definizione di policy semantica per la privacy
   - stabilire cosa filtra `-z`
   - stabilire cosa resta sempre visibile
   - mantenere comportamento coerente in tutte le sezioni

## Sequenza proposta

### Fase S1 - Validazione funzionale

- confronto con `inxi` su host reali
- apertura issue list interna
- chiusura delle differenze critiche

### Fase S2 - Stabilizzazione contratto

- freeze schema JSON
- freeze opzioni CLI supportate
- freeze warning codes principali

### Fase S3 - Release candidate

- test verdi
- clippy verde
- golden verdi
- sample reali verdi
- documentazione minima pronta

### Fase S4 - `v0.1.0`

- tag della prima versione stabile
- da quel punto, nuove sezioni solo come incremento compatibile

## Stima pragmatica

Se manteniamo il perimetro attuale, la prima versione stabile non e' lontana.

La considero raggiungibile dopo:

- `1` ciclo serio di confronto con `inxi`
- `1` ciclo di correzione sui collector
- `1` ciclo di freeze su CLI/JSON/warning
- `1` ciclo finale di test e documentazione

Tradotto: non mancano "mesi di architettura", mancano soprattutto `4` blocchi di consolidamento.

## Backlog post-stable

Da lasciare fuori da `v0.1.0` salvo necessita' forti:

- `Audio`
- `Battery`
- `USB`
- `Sensors`
- `Bluetooth`
- `RAID`
- `Repos`
- `Processes`
- frontend `TUI`

## Nota progettuale

La `TUI` va trattata come frontend successivo sopra il core gia' stabilizzato.

Decisione corretta per adesso:

- prima stabilizzare `collector + model + renderer base`
- poi valutare la `TUI` senza contaminare il core con vincoli di presentazione
