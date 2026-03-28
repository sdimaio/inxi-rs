# inxi-rs

`inxi-rs` e' un tool di system information per Linux scritto in Rust, pensato
per restare `read-only` e liberamente ispirato a
[`inxi`](https://github.com/smxi/inxi).

Non e' una traduzione letterale del codice Perl e non e' un port ufficiale.

L'obiettivo del progetto e':

- mantenere un core riusabile e separato dalla CLI
- offrire output `screen` e `json` stabili
- rendere espliciti fonti dati, warning e fallback
- preparare il terreno per una futura TUI senza rifare il motore

Il progetto e' mantenuto da Simmaco Di Maio ed e' anche un investimento tecnico
e personale su Rust come linguaggio di lungo periodo.

## Stato del Progetto

Versione attuale: `0.1.0-alpha.1`

Questo significa:

- il progetto ha gia' una struttura professionale
- e' gia' utile su Linux
- non va ancora presentato come equivalente completo a `inxi`
- il contratto CLI/JSON e alcune sezioni stanno ancora maturando

## Perché Esiste

Il valore di `inxi` non sta nel Perl in quanto tale, ma in alcune idee molto
forti:

- sezioni pragmatiche
- output utile in assistenza e diagnostica
- fallback robusti
- filtri privacy

`inxi-rs` nasce per portare queste idee in una architettura Rust piu' pulita:

- stato meno globale
- collectors modulari
- modello dati esplicito
- renderer separati
- base piu' adatta a una futura TUI

## Perché Rust

Rust e' stato scelto per due motivi:

- e' molto adatto a modellare dati di sistema parziali, opzionali e con fallback
- e' il linguaggio su cui il maintainer vuole investire nel lungo periodo

Dettagli in [docs/it/perche-rust.md](docs/it/perche-rust.md).

## Perimetro Attuale

Supportato oggi:

- Linux-only
- output `screen` e `json`
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

Non ancora obiettivo della release corrente:

- piena parita' con `inxi`
- BSD e altri sistemi
- TUI
- ampia compatibilita' con tutte le opzioni storiche di `inxi`

## Modello di Sicurezza

`inxi-rs` e' progettato per restare osservativo.

Regole attuali:

- nessuna scrittura su filesystem
- nessuna scrittura su `/proc`, `/sys`, `/dev`
- nessuna shell
- nessuna privilege escalation
- nessun accesso di rete
- comandi esterni solo se auditati e in whitelist

Dettagli in [docs/it/sicurezza.md](docs/it/sicurezza.md).

## Avvio Rapido

Esecuzione base:

```bash
cargo run -p inxi-cli -- -b
```

Output JSON:

```bash
cargo run -p inxi-cli -- -b --output json
```

Diagnostica collector:

```bash
cargo run -p inxi-cli -- --self-check
```

## Documentazione

- [Indice Documentazione](docs/it/index.md)
- [Panoramica](docs/it/panoramica.md)
- [Sicurezza](docs/it/sicurezza.md)
- [Roadmap](docs/it/roadmap.md)

## Licenza

Il progetto e' distribuito con licenza `GPL-3.0-or-later`.

Vedi [LICENSE](LICENSE).
